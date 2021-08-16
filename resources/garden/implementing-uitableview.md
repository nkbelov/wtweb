# Implementing UITableView

## Chapter 1: the naïve beginnings

When I just started writing [Undebit](https://apps.apple.com/us/app/undebit/id1569472635), I had an idea to use a table view cell animation similar to what one of my favourite apps ever, [Things 3](https://culturedcode.com/things/), has. I discovered quite soon that implementing such animation with the native `UITableView` was either impossible or incredibly burdensome, and simultaneously I noticed that Things 3 doesn’t use `UITableView` — they have their custom implementation (besides their highly specific design, there is a subtle visual bug that clearly indicates they're using custom code). Being adventurous as I usually am, I wrote a custom one for my app too — and even though, ironically, I decided not to use the animation I originally planned, this custom implementation eventually replaced all occurrences of `UITableView` within Undebit. 

This was quite a journey for a couple of reasons: because writing custom UI is deemed expensive, if your needs are already mostly covered by UIKit, there's not much reason for you to roll out a complicated component — but this also means that if people do it rarely, they write *about* it even more rarely, too, so the information I went off was extremely scarce. Second, UIKit relies on a lot of not only undocumented behaviour, but also private APIs that are simply inaccessible to us.

Still, I’m writing this to show several things: first, there’s not much stuff in the UIKit that you *have* to rely upon. You really only have to use `UIView` for obvious reasons and perhaps `UIScrollView` because you don’t want to spend ages on guessing the deceleration constants that make it feel like the native one. Second, writing custom stuff is not *that* expensive. It will be a leap when you’re dealing with it for the first time because the documentation is extremely lacking, but if you understand how UIKit’s internals work, writing custom UI can turn from a struggle into a rewarding routine. As such, this is probably less of a normal tutorial than an exploration document, aiming to list different considerations one might go into when writing their custom UI. Nothing written here is prescriptive — I provide my reasoning behind the choices I make, but it's exactly the point of writing custom code to deviate from some common behaviour if your implementation needs it.

In reality, list views (and I will use the term “list view” instead of “table view” henceforth) are not complicated components. Yes, if you look at *all* of the features of `UITableView`, like VoiceOver support or drag-and-drop, it might seem daunting to think about the volume of work that needs to be put into it — but the core logic is actually quite simple. The implementation that Undebit uses at this point is really only 1250 lines of code large, including lengthy comments and assertion tests — and while that **might** sound like a somewhat large number, it is nowhere close to the amount of code that typical projects have, and it indeed is just a minuscule fraction of Undebit’s codebase. Lastly, the complexity of UIKit's components is to a significant part due to their legacy, sometimes poor design and the need to ensure backward compatibility. If you're able to write everything from scratch, you also have the power to ensure simplicity.

The overall idea behind `UITableView` is very simple: you want to display a scrollable list of rows. The reason, however, why something like a simple `UIStackView`  embedded into a `UIScrollView` cannot be used (and thus the reason why `UITableView` is, above almost all others, the core component of many, many apps) is because a `UIStackView` with, say, 1000 subviews will keep all of them in its view hierarchy — and this is **a lot** of memory. If each of your rows is 50 points tall and 375 points wide (which corresponds to 75000 pixels on a 2x Retina display), then the row will require 300KB worth of pixel data, as each needs 4 bytes to store its RGBA values. A thousand of these rows will thus need whopping 300MB of RAM — just to store a huge amount of visual data that will never be shown simultaneously.

That’s why rows are reused, and that’s why you `dequeueReusableCell` to make it show in a `UITableView`. Whenever the user scrolls far enough that a cell goes off screen, it is placed into a reuse queue, from which it is retrieved again shall there be a need to display another cell with the same reuse identifier[^reuseid].

So, architecture-wise, for our custom implementation, we need:

- A `UIScrollView` as the base view in which we put our cells,
- A way to tell our list view which row classes we are going to use,
- A way to track when a row goes onto the screen and when it leaves it,
- Some storage (I’ll call it a reuse pool) to put rows for later reuse,
- And finally, a way to lay out our rows on screen.

That’s kinda it. The basic functionality of showing some scrollable content only depends on these five components, the first one already being provided to you by UIKit. As you will see, the other four pieces are not at all hard to implement, and we can fit them into just 250 lines of code.

To top everything off, here’s a very professional illustration showing what's supposed to happen when the user scrolls through the content:
![Reuse process](/images/dequeue-1.png)

### Meet `layoutSubviews()`
I’m not old enough to have started programming at the time when UIKit was just an innocent baby, so if you, like myself, became an iOS developer in the recent years, the drill has been such: “Auto Layout”. That’s how you would work with 95% of your views, if not more, and you might even try to push it to 100% if that’s the only way you know to lay out your views.

But pre-iOS5, there was no Auto Layout, so you would need to specify the exact position of the view rectangle on the screen by setting its coordinates and size, which sometimes would even require quite a bit of math. And — what a coincidence — components like list views fall exactly into the 5% bucket of cases where using Auto Layout is more of a hindrance than the right tool for the job. It would be still possible to use it, but I expect it to be highly inconvenient — and the performance would suffer, too.

As such, we will work with `layoutSubviews()` directly. It’s a method on `UIView` that you override to manually set the positions and sizes of your views. It is actually this same method — when not overridden — that internally triggers the Auto Layout engine and makes it figure out the positions and sizes of subviews on its own. Here, we will write out own layout logic that instead performs this task manually.

You are not supposed to call `layoutSubvews()` yourself (although if you do, usually nothing really terrible happens). Instead, it is called by the UIKit if the internal `needsLayout` flag is set: each frame, which is 60 or 120 times per second, depending on the device, UIKit will traverse the view hierarchy, look for this flag on the views and call `layoutSubviews()` if they have it set. This helps with performance and conserves the battery, as only views that explicitly signal that they have to perform a layout pass will be asked do it, and only do it once per frame.

To signal that a view wishes to perform a layout pass, `setNeedsLayout()` is called. You can call this method yourself is something within a view's state dictates that its subviews need to be rearranged (for example, in response to a `UIGestureRecognizer`'s callbacks). But more importantly, this function will also be called automatically on certain fundamental events: for example, when the view has just been added to the hierarchy and needs to perform its first layout pass — or, of most interest to us — when its `bounds` change.

The scrolling behaviour of `UIScrollView` is achieved by shifting the “window” through which you look at its contents, and when a user scrolls up or down, this window gets shifted vertically to reveal different parts of the scroll view’s coordinate plane, which holds its subviews. It is exactly the `bounds` property that specifies `UIScrollView`'s window position and size, and the system calls `setNeedsLayout()` each time either the position or size parameters change.

This means that with each tick of a scrolling interaction, we are given a chance to re-consider what we are showing inside the scroll view, and this exact chance is what we will use to track down the rows that are not visible anymore to reuse them. This is also where we will see if there are some views that are about to appear — in which case we will take a hidden view from the reuse pool (or instantiate a new one if there are none left) and position it on screen.

### First steps
You can find the Xcode project for this post on [my GitHub](https://github.com/wtedst/ListView). Each chapter receives a corresponding branch, so you may freely switch between them to follow the incremental implementation of the features. If you download and build from the `base` branch of the project, you should be able to see a teal view on the screen, slightly inset from its edges — this is going to be our initial debugging setup. You may then follow along by reading the code here and putting it into the appropriate places — or checkout the `step-1` branch to see the final result of this chapter.

All the work we are going to do will be done in `ListView.swift`. At the beginning, it’s merely an empty subclass of `UIScrollView`. We will now implement the basic logic that will allow us to create a list of rows of constant height that will be reused on scroll.

Like `UITableView` itself, we will organise our layout into sections and rows. There’s no strict reason for doing this, as we could as well build everything as a one-dimensional list (or, on the other hand, we could have gone with a three-dimensional structure with rows, sections and super-sections) — the decision is up to you. I will just try to stick with `UITableView`’s API as closely as possible for demonstration purposes.

So, let’s finally do some work. As I just discussed, we will need some properties on `ListView` that will store information about the number of rows and sections and their position on the screen. We also will need to have the reuse pool, which also has to differentiate between the different view classes that appear as rows. To support these, we will also declare a couple of helpful data types:

```swift
/// A structure to store the layout information of each row in a `ListView`.
/// Since the rows are always stretched to fill the full width of the `ListView`, we do not have
/// to record their x-position or their width. All that matters to us is knowing where they are located
/// on the vertical axis — and thus we only need their `y` coordinate and their `height`.
/// We mainly need this to 1. use less memory and 2. be more cache-friendly on searches.
struct Vertical {
    
    var y: CGFloat
    var height: CGFloat
    
    /// A convenience property to calculate the position of the bottom edge of the row
    var maxY: CGFloat { y + height }
    
    init(y: CGFloat, height: CGFloat) {
        precondition(height >= 0)
        self.y = y
        self.height = height
    }
    
    /// This function will be used to test if a view with a given `Vertical` is not inside the `bounds`
    /// of the `ListView` anymore and thus needs to be reused.
    func intersects(_ other: Vertical) -> Bool {
        // Two verticals intersect if none of the two coordinates (y and maxY)
        // of one lie completely to one side of the other vertical
        //            v~~~above      v~~~below
        return !(maxY < other.y || y > other.maxY)
        
    }
    
    func intersects(_ rect: CGRect) -> Bool {
        return intersects(rect.vertical)
    }
}

extension CGRect {
    
    /// A convenience property to extract a `Vertical` from views' frames
    var vertical: Vertical {
        return Vertical(y: minY, height: height)
    }
    
}

final class ListView: UIScrollView {
    
    static let defaultRowHeight: CGFloat = 50
    
    /// The index of a row, corresponds to `IndexPath`.
    struct Index: Hashable {
        
        var section: Int
        var row: Int
        
        init(section: Int, row: Int) {
            self.section = section
            self.row = row
        }
    }
    
    /// The dimensions of the list. The position in the array corresponds to a section,
    /// and the integer entry corresponds to the number of rows in that section.
    private(set) var dimensions: [Int] = []
    
    /// The set of currently displayed rows. Rougly corresponds to `visibleCells` and `indexPathsForVisibleRows`,
    /// except that now it is one dictionary.
    private(set) var displayedRows = [Index: RowView]()
    
    /// Stores the layout information of each individual row.
    private var verticals = [Index: Vertical]()
    
    /// The reuse pool. The key is an object identifier corresponding to the dynamic type of a row. When rows are reused,
    /// they need to be put into the appropriate array, so that when we retreive them later, we get a view of the right type.
    /// The key is thus equivalent to a `reuseIdentifier` within `UITableView`.
    private var pool = [ObjectIdentifier: [RowView]]()

    /// This is the equivalent of `cellForRow(at:)` method in `UITableViewDataSource`.
    private var rowViewSource: Optional<(Index, ListView) -> RowView> = nil

    init() {
        super.init(frame: .zero)
        alwaysBounceVertical = true
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

extension ListView.Index: Comparable {
    
    /// List view indices are totally ordered, meaning they always compare as `<`, `=` or `>`, and thus are `Comparable`.
    /// An index is smaller if it appears in an earlier section or if it appears earlier in the same section.
    static func < (lhs: ListView.Index, rhs: ListView.Index) -> Bool {
        lhs.section < rhs.section || (lhs.section == rhs.section && lhs.row < rhs.row)
    }
}

```

I tried to make the documentation comments as self-explanatory as possible, but let's talk through this stuff one more time. We first declare a struct `Vertical`, which will be our primary means of remembering the position of rows that are *not* currently on screen. We could have gone with `CGRect`, but there's not much reason in storing the x-coordinate and width of a row, as rows will always be stretched horizontally to fill the list view. As such, it's easier to reason about our layout if we simply remove the unnecessary data. Plus, this helps with memory use and performance if our list view has a ginormous amount of rows, since thinner storage is more cache-friendly. Also notice the convenience extension on `CGRect` that will allow us to extract a `Vertical` from it.

Second, in `ListView` itself, we declare a `ListView.Index` struct. `UITableView` uses `IndexPath`, which is theoretically a data type that can store a path of indices (hence the name) of infinite depth, like *"section 0, row 2, sub-item 15, sub-sub-item 1"* and so on. However, there's little use for this in our case and I have never really seen any app using `IndexPath`s outside of indexing `UITableView` or `UICollectionView` items. So, again, we will work with a simpler model. Notice that at the very bottom, I declared a `Comparable` conformance for `ListView.Index` as well.

Finally, within `ListView`, we declare the storage we need for all the various data that has to be recorded to keep track of section sizes (the `dimensions` array), currently displayed rows (`displayedRows`) and the overall layout of the list view, which maps an `Index` to a `Vertical`, indicating where a particular row is located. The reuse `pool` is a dictionary mapping the type of a view that we can get with `type(of:)` at runtime to an array of currently unused views that can be dequeued[^deq] and made visible again.

Ignore `rowViewSource` for now; its purpose will be disclosed more clearly later (although the comment kinda gives it away already).

You may also notice the `RowView` type. It's a protocol constrained on `UIView` (so theoretically, every `UIView` subclass will be able to conform to it) — no need for special classes like `ListViewCell`! I intentionally don't show its declaration now, as we will discover the required methods later.

### The layout algorithm

Here goes our overridden `layoutSubviews()`:

```swift
override func layoutSubviews() {
    // Apple documentation states that we have to call `super` for internal bookkeeping.
    // Because this triggers Auto Layout, do it as early as possible so that our manual
    // layout stays unaffected
    super.layoutSubviews()
    
    // Query the indices of rows that are wisible within the current bounds
    // Note that the current way is suboptimal: we are performing a linear search on ordered data;
    // at some point, we will refactor this to use a binary search, which is O(log n) instead of O(n)
    let visibles: [(Index, Vertical)] = verticals.filter { $0.value.intersects(bounds) }
    
    // Reuse the views that are disappearing. Doing this first lets us reuse them right away
    // if a view of the same class reappears with a different index.
    for index in displayedRows.keys {
        if !visibles.contains(where: { $0.0 == index }) {
            reuse(at: index)
        }
    }
    
    assert(displayedRows.keys.allSatisfy { index in visibles.contains { $0.0 == index } },
            "Not all hidden rows were reused!")
    
    // Lay out the visible rows: fetch them, calculate the frame and assign it
    for (index, vertical) in visibles {
        let view = getView(for: index)
        let frame = frame(for: vertical)
        
        // This is a small optimisation: because setting `bounds` on a view calls `setNeedsLayout` on it
        // without this check all our rows would issue a layout pass on each scroll tick
        // — bad for performance!
        if view.frame != frame {
            view.frame = frame
        }
    }
    
    assert(visibles.allSatisfy { displayedRows.keys.contains($0.0) }, "Not all visible rows are actually displayed!")
    assert(displayedRows.allSatisfy { $0.value.isHidden == false }, "Not all visible rows are unhidden!")
}
```

I want you to appreciate how *incredibly* simple it is[^simple]. Yes, it uses some helper methods that I will disclose shortly, but the overall logic involves just three steps:

1. Find rows that are going to be visible in our current `bounds` by testing if a given `Vertical` intersects it,
2. Hide and reuse the rows that are not visible anymore,
3. Retrieve a view for each visible row and assign it its frame,
4. Done!

As I discussed earlier, this method will be called each time the user scrolls the list and thus `bounds` get shifted up or down the coordinate system of the list view. This means that we are able to react quickly and reuse views as soon as they are allowed to disappear, thus keeping the amount of created views as low as possible. On the other hand, this means that this function will be called very often — very likely every frame — which means that this function has to be extremely efficient at what it's doing. As I already have noted in the comments, we are currently using a very inefficient search strategy[^ineffsearch], which we will need to take care of later. 

## The helper methods

Here you can see all the helper methods we need to get views from the reuse pool onto the screen and back:

```swift
/// Convenience function to convert a `Vertical` into a rectangle spanning the list view horizontally.
private func frame(for vertical: Vertical) -> CGRect {
    return CGRect(x: bounds.minX,
                  y: vertical.y,
                  width: bounds.width,
                  height: vertical.height)
}

/// Reuse a displayed view: remove it from `displayedRows` dictionary,
/// hide it and put into the appropriate reuse pool array.
private func reuse(at index: Index) {
    let view = displayedRows[index]!
    view.isHidden = true
    displayedRows[index] = nil
    
    let type = type(of: view)
    let poolKey = ObjectIdentifier(type)
    
    assert(pool[poolKey] != nil, "Should create a pool for \(poolKey) when first dequeuing the row")
    pool[poolKey]!.append(view)
}

/// This function either returns a row that is already visible at this index, or asks the `rowViewSource` for a new one.
private func getView(for index: Index) -> RowView {
    guard displayedRows[index] == nil else { return displayedRows[index]! }

    let view: RowView = rowViewSource!(index, self)
    
    view.isHidden = false
    view.autoresizingMask = []
    // Because we perform layout manually, we need to explicitly re-enable this property
    view.translatesAutoresizingMaskIntoConstraints = true
    
    displayedRows[index] = view
    
    return view
}

/// The equivalent of `dequeueReusableCell`, except that we use the more robust generic API.
func dequeueRow<V: RowView>(type: V.Type, at index: Index) -> V {
    let poolKey = ObjectIdentifier(V.self)
    
    let view: V
    if pool[poolKey] == nil {
        // Just create an empty pool array for this type for later
        pool[poolKey] = []
        view = V()
    } else if pool[poolKey]!.isEmpty {
        // The pool exists but has been exhausted — need to create a new row view anyways
        view = V()
    } else {
        // The pool has a view waiting to be reused
        view = pool[poolKey]!.popLast()! as! V
    }
    
    addSubview(view)
    return view
}
```

Let's tackle them one at a time:

- `frame(for:)` converts a `Vertical` into a `CGRect` that spans the list view horizontally — nothing complicated here,
- `reuse(at:)` takes an index of a displayed row, hides it, de-registers as a displayed row and adds it to the appropriate array within the reuse pool. Because this is our internal API, we are in control of calling it correctly — hence the assertions and force-unwraps within the method,
- `getView(for:)` is perhaps the most important one. Whenever it's passed an index of a row that's already displayed, it just returns this row. Otherwise, it will query our equivalent of `tableView(_:cellForRowAt:)` — a function with the `(Index, ListView) -> RowView` signature. Because of the `guard` statement at the top, this will only be called once for a view that's just about to appear, giving the client side a chance to populate the view with the appropriate data — as you would expect with a normal `UITableView`,
- `dequeueRow(type:at:)` is how the implementation of `rowViewSource` will be able to communicate with our reuse pool and ask it for a view of the desired type to populate.

You may notice in `dequeueRow(type:at:)` that we create the views ourselves, by calling `V()` (an empty initializer), on demand. Because `ListView` is the entity managing the reuse pool, and our goal is to bring the amount of subviews in `ListView` to the absolutely necessary minimum, we want to control the circumstances in which views are created. Of course, then, because we are situated within a generic function, the ability to call this empty initializer has to be guaranteed by some sort of a protocol — since there's generally no guarantee that some user-supplied `UIView` subclass will provide this initialiser — which is exactly why we introduce the `RowView` protocol. In fact, the empty initializer is the only requirement we have to pose:

```swift
protocol RowView: UIView {
    init()
}
```

Later on we might want to add additional methods with a default implementation to supply more API similar to that of `UITableViewCell`, like `prepareForReuse()`, but otherwise there's nothing else we have to ask of a view class that wants to be able to displayed as a row — it just has to be constructible by our own means.

Finally, let's discuss the last method, which kickstarts this whole thing:

```swift
/// The main API to `ListView`: it registers the initial sizes of the sections and supplies the closure
/// which will be used to populate the rows as they appear on screen.
/// This is equivlent to setting a `dataSource` of `UITableView`.
func reload(dimensions: [Int], rowViewSource: @escaping (Index, ListView) -> RowView) {
    self.rowViewSource = rowViewSource
    
    // First, hide all views that we are displaying already
    for index in displayedRows.keys {
        reuse(at: index)
    }
    
    assert(displayedRows.isEmpty)
    
    let rowCount = dimensions.reduce(0, +)
    
    verticals.removeAll(keepingCapacity: true)
    verticals.reserveCapacity(rowCount)
    
    // Populate `verticals` with the new layout.
    // This is just rows stacked on top of each other
    var currentY: CGFloat = 0
    for section in dimensions.indices {
        for row in 0..<dimensions[section] {
            let index = Index(section: section, row: row)
            let vertical = Vertical(y: currentY, height: ListView.defaultRowHeight)
            verticals[index] = vertical
            currentY = vertical.maxY
        }
    }
    
    assert(verticals.count == rowCount)
    
    self.dimensions = dimensions
    
    // Reset the scroll location to top
    bounds.origin = .zero
    contentSize.height = currentY
    setNeedsLayout()
}
```

This method is used as the entry point to `ListView`, and its invocation does similar things as giving a `UITableView` its `dataSource()`: we supply the initial dimensions of our dataset and a closure that we already seen being used in `getView(for:)`. `ListView` also discards any previous layout if it existed and re-builds the new layout with a default row height. Lastly, it resets the scroll position (i.e. `bounds`) by shifting it to the top edge of the view. Because at this point we also already know the total scrollable height of the content, we set `contentSize.height` here as well.

I chose not to purge the reuse pool for simplicity as, in practice, when reloading a list view, we don't want anything dramatically different — we usually just want to show different data which is still of the same kind — and thus likely to use the same row view types. If you wish, though, you can totally remove all the subviews from the reuse pool by calling `removeFromSuperview()` on them first and then erasing the whole pool dictionary.

That's it! The first stage is done. You may now navigate to `ViewController.swift` and add some setup code:

```swift
import UIKit

class ViewController: UIViewController {

    private let listView = ListView()
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        view.addSubview(listView)
        listView.backgroundColor = .systemTeal
        
        // Setting this property to `false` lets you see rows being reused in action!
        listView.clipsToBounds = true
        
        listView.reload(dimensions: [100, 200]) { index, listView in
            let label = listView.dequeueRow(type: UILabel.self, at: index)
            label.backgroundColor = .systemGreen
            label.text = "Section: \(index.section), row: \(index.row)"
            return label
        }
    }
    
    override func viewWillLayoutSubviews() {
        listView.frame = view.bounds.inset(by: view.safeAreaInsets).insetBy(dx: 16, dy: 24)
    }

}

extension UILabel: RowView { }
```

The beauty of our implementation is that we can use `UILabel` as our row type directly by writing an empty extension conforming the class to `RowView`. If you run the code now, you will see green rows labeled with their respective section and row indices. To prove that we aren't creating 300 subviews, add `print(subviews.count)` at the end of `layoutSubviews()` of `ListView`. You will see that the number of subviews is just enough to cover the screen, and remains constant as the user scrolls.

---
*This is an ongoing series. The next chapter will discuss various optimisations for the layout pass.*

---
[^reuseid]: In practice, most people just make a reuse ID for each `UITableViewCell` subclass, simply tying it to the class name itself. On this note: our implementation will also try to address some places in `UITableView`’s API design which do not align with how it's normally used, especially given Swift's current capabilities.

[^deq]: In reality, there's no need to regard the reuse pool as a queue. We will take unused views from the end of the array, not the front.

[^simple]: Well, it's only this simple for now. More complex features, such as using Auto Layout to dynamically calculate row heights (the behaviour enabled by `UITableView.automaticDimension`) will introduce additional complexity.

[^ineffsearch]: If you finished reading the first chapter, you may locate the setup code in `ViewController.swift` and try creating a list view with a very generous amount of rows. Observe how the performance drops significantly when the total number of rows approaches ten thousand or even hundred thousand.