use std::{path::PathBuf, fs::{read_to_string}, collections::HashMap};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::render::{extract_meta, render_markdown};

#[derive(Deserialize)]
struct ManifestEntry {
    name: String,
    published: bool
}

#[derive(Deserialize)]
struct PostsManifest {
    posts: Vec<ManifestEntry>
}

impl PostsManifest {

    pub fn load() -> Self {
        let path = PathBuf::from("posts/posts.toml");
        toml::from_str(&read_to_string(path).unwrap()).unwrap()
    }

}

impl<'a> IntoIterator for &'a PostsManifest {
    type Item = &'a ManifestEntry;
    type IntoIter = <&'a Vec<ManifestEntry> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.posts.iter()
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Post {
    pub title: String,
    pub abs: Option<String>,
    pub text: String,
    pub published: bool
}

impl Post {

    fn try_from(path: &std::path::Path) -> std::io::Result<Self> {
        let markdown = read_to_string(path)?;
        let (title, abs) = extract_meta(&markdown);
        let text = render_markdown(&markdown);
        Ok(Post {
            title,
            abs,
            text,
            published: false
        })
    }
}

#[derive(Debug)]
pub struct Posts {
    post_list: Vec<String>,
    posts: HashMap<String, Post>,
    images: HashMap<String, Bytes>
}

impl Posts {

    pub fn load() -> Self {
        let manifest = PostsManifest::load();
        let mut posts = HashMap::new();
        let mut images = HashMap::<String, Bytes>::new();
        let mut path = PathBuf::from("posts");
        let mut post_list = Vec::new();

        for entry in &manifest {
            path.push(&entry.name);
            if path.is_dir() {
                let Ok(read_dir) = path.read_dir() else {
                    println!("While loading from manifest: skipped dir {:?}, couldn't read", &path);
                    continue;
                };

                for item in read_dir.filter_map(|i| i.ok()) {
                    if item.file_type().unwrap().is_dir() {
                        println!("While searching dir {:?}, skipping subdir {:?}", &path, &item.file_name());
                        continue;
                    }
                    
                    let filename = item.file_name().to_str().unwrap().to_owned();
                    // FIXME: Update when `is_some_and` is available.
                    if filename.ends_with(".md") {
                        if let Ok(mut post) = Post::try_from(&item.path()) {
                            post.published = entry.published;
                            posts.insert(entry.name.clone(), post);
                            post_list.push(entry.name.clone());
                        }
                    } else {
                        // Assumed to be an image, but can really be anything else.
                        let compound_name = entry.name.clone() + &filename;
                        let Ok(bytes) = std::fs::read(&path) else { continue; };
                        let bytes = Bytes::from(bytes);
                        images.insert(compound_name, bytes);
                        // TODO: Impl alerting if img already present
                    }
                }
            } else {
                // Should be just a single post with this name;
                // the path doesn't contain the extension yet, so set it.
                path.set_extension("md");
                if let Ok(mut post) = Post::try_from(&path) {
                    post.published = entry.published;
                    posts.insert(entry.name.clone(), post);
                    post_list.push(entry.name.clone());
                }
            }

            path.pop();
        }

        // Reverse because the latest posts get appended to the bottom.
        post_list.reverse();

        Self {
            post_list,
            posts,
            images
        }
    }

    pub fn get_post(&self, name: &str) -> Option<&Post> {
        self.posts.get(name)
    }

    pub fn previews(&self) -> Vec<Post> {
        self.post_list.iter().take(3).map(|name| self.posts.get(name).unwrap().clone()).collect()
    } 

}