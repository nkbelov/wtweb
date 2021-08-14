# Implementing a UITableView

List views are basically omnipresent in apps. 

Because these views really display their items as a list, not as a table, I will go on with using the term *list view* in this post.

If you have used Things, you are very likely aware of its unique feel. I can say with quite a great certainty that they use their own implementation of list views — I once spotted a very subtle bug that indicated it was a custom component. 

In reality, list views are not complicated components. Yes, if you look at all of the features of UITableView, like VoiceOver support or drag-and-drop, it might seem daunting to think about the volume of work that needs to be put into it, but the core logic is actually quite simple. The implementation that Undebit uses at this point is really only 1250 lines of code large, including lengthy comments and assertion tests — and while that *might* sound like a somewhat large number, it is nowhere close to the amount of code that typical projects have, and it indeed is just a minuscule fraction of Undebit’s codebase.

The overall idea is very simple: you want to display a scrollable list of rows. The reason, however, why something like a simple `UIStackView`  embedded into a `UIScrollView` cannot be used (and thus the reason why `UITableView` is, above almost all others, the core component of many, many apps) is because a `UIStackView` with, say, 1000 subviews will keep all of them in its view hierarchy — and this is *a lot* of memory. If each of your rows is 50 points tall and 375 points wide (which corresponds to 75000 pixels on a 2x Retina display), then the row will require 300KB worth of pixel data, as each needs 4 bytes to store its RGBA values. A thousand of these rows will thus need whopping 300MB of RAM — just to store a huge amount of visual data that will never be shown simultaneously.

That’s why rows are reused, and that’s why you `dequeueReusableCell` to make it show in a `UITableView`. Whenever the user scrolls far enough that a cell goes off screen, it is placed to a reuse queue, from which it is retrieved again shall there be a need to display another cell with the same reuse identifier (in practice, I think, people just make a reuse ID for each `UITableViewCell` subclass). On this note: our implementation will also try to address some places in `UITableView`’s API design which feel clunky and, frankly, not-so-well-designed. 

So, architecture-wise, for our custom implementation, we need:

- A `UIScrollView` as the base view in which we put our cells,
- A way to tell our list view which row classes we are going to use,
- A way to track when a row goes onto the screen and when it leaves it,
- Some storage (I’ll call it a reuse pool) to put rows for later reuse,
- And finally, a way to lay out our rows on screen.

That’s kinda it. The basic functionality of showing some scrollable content only depends on these five components, the first one already being provided to you by UIKit. As you will see, the other four pieces are not at all hard to implement, and we can fit them into just 350 lines of code.

## Meet `layoutSubviews()`
I’m not old enough to have started programming at the time when UIKit was just an innocent baby, so if you, like myself, became an iOS developer in the recent years, the drill has been such: “Auto Layout”. That’s how you would work with 95% of your views, if not more, and you might even try to push it to 100% if that’s the only way you know to lay out your views, even if it’s not the best tool for the job — just because you don’t know otherwise; because the other way is not really taught anymore.

But pre-iOS5, there was no Auto Layout, so you would need to specify the exact position of the view rectangle on the screen by setting its coordinates and size, which sometimes would even require quite a bit of math. And — what a coincidence — not only I am a mathematician, but also components like list views fall exactly into the 5% bucket of cases where using Auto Layout is more of a hindrance than the right tool for the job. It would be still possible to use it, but I expect it to be highly inconvenient — and the performance would suffer, too.

So finally, meet `layoutSubviews()`. It’s a method on `UIView` that you override to manually set the positions and sizes of your views. It is actually the method — when not overridden — that internally triggers the Auto Layout engine and makes it figure out the positions and sizes of subviews on its own. Here, we will write out own layout logic that 

You are not supposed to call `layoutSubvews()` yourself (although if you do, nothing really terrible happens). Instead, it is be called by the UIKit each frame if the internal `needsLayout` flag is set: basically, each frame, which is 60 or 120 times per second, depending on the device, UIKit will traverse the view hierarchy, look for this flag on them and call `layoutSubviews()`. This is simply done for performance and battery reasons: most of the time the content of our views remains the same, so there’s no real need in re-drawing the same content each frame. 

```swift
override func layoutSubviews() {
    /*
        The idea behind the process of layout with reusable views is to get
        "snapshots" of the current situation within the current `bounds`.
        
        Which in itself is not a tricky part — simply filtering a frame cache
        would do the job. The actual problems arise when we want to do more complex things,
        like querying the rows' sizes dynamically or performing insertion/deletion animations.
        
        At initialisation, `ListView` simply assumes that all rows have the same default height
        and creates the layout; the actual height of any row is unknown at this point. Whenever a
        row appears which hasn't yet been asked for its size (that is, its entry in
        the `heights` dictionary is empty), we populate the row view and ask it for the size given the data
        that it's about to display.
        
        If its actual height differs from the default (which is a very likely event), it affects the
        `y` coordinates of views below it. In case a row is larger than expected, the views below need to be
        pushed down; if it's shorter — the views below need to be pulled up. This is achieved
        by changing their `y` coordinate by the difference. The latter case also poses another
        problem: more rows could've become visible than we assumed. In this case, we need to requery
        the layout cache for those.
        
        Insertions, deletions and moves mess up not only with the layout state
        (we need to compose a "from" snapshot and a "to" snapshot to animate between them),
        but also remap views and data to new indices. Actually, they remap whole ranges: for instance,
        insertion into the index (0,0) would remap rows at (0,1), (0,2) etc. into (0,2), (0,3) etc.
        
        Insertions, deletions and moves can also be simultaneous: the net effect of deleting and then inserting
        a row at the same index is not the index shifting just mentioned above, but instead there will be
        *two* rows simultaneously associated with the insertion/deletion index, which need to be animated
        appropriately. In this case, there should be a proper mechanism of associating multiple views
        with the same index when building "from" and "to" snapshots.
        
        `List view` does not handle selection and highlighting of rows as it's a purely cosmetic interaction
        that can be provided by row views themselves.
        */

    super.layoutSubviews()

    // `headerViewHeight == nil` signifies that we are laying header out for the first time
    if let header = headerView, headerViewHeight == nil {
        addSubview(header)
        let height = header.systemLayoutSizeFitting(.zero).height
        headerViewHeight = height
        header.translatesAutoresizingMaskIntoConstraints = true
        // Align to header. This only happens once as `headerViewHeight == nil` is only true on first layout
        contentOffset.y = -height
    }

    if let header = headerView {
        let headerFrame = CGRect(x: bounds.minX,
                                    y: -headerViewHeight!,
                                    width: bounds.width,
                                    height: headerViewHeight!)
        
        if header.frame != headerFrame {
            header.frame = headerFrame
            contentInset.top = headerViewHeight! + additionalTopInset
        }
    } else {
        contentInset.top = additionalTopInset
    }

    let layout = mainLayoutLoop(on: &verticals,
                    dimensions: dimensions,
                    displayedRows: displayedRows.keys,
                    for: bounds,
                    heightSource: queryRowHeight(for:))

    if !isUpdating {
        // If we are not updating, reuse disappearing (previously visible, but not in the current layout) rows
        for i in layout.disappearing {
            assert(displayedRows[i] != nil)
            reuseView(displayedRows[i]!, with: i)
            // Note that this is the only place we remove something from displayed rows
            displayedRows[i] = nil
        }
    } else {
        // Layout every view that was displayed. Views that are in `layout.disappearing` still need to be animated out
        for (i, view) in displayedRows {
            let vertical = verticals[i]!
            let frame = frameForVertical(vertical)
            view.frame = frame
        }
    }

    for i in layout.visible.keys {
        
        let view: UIView
        if layout.appearing.contains(i) {
            view = getView(for: i)
            // Note that this is the only place we add the view to `displayedRows`
            displayedRows[i] = view
        } else {
            view = displayedRows[i]!
        }
        
        let vertical = verticals[i]!
        view.frame = frameForVertical(vertical)
        view.translatesAutoresizingMaskIntoConstraints = true
        
        #if DEBUG
        if let v = view as? Tester {
            v.debugDisplay(index: i)
            v.debugDisplayFrame()
        }
        #endif
    }


    contentOffset.y += layout.offsetAdjustment
    contentSize.height += layout.heightAdjustment
    verticalScrollIndicatorInsets.top = additionalTopInset - safeAreaInsets.top
    // TODO: We actually do not really account for width change,
    // since we never reask the rows for preferred height given new width
    // IDEA: A reasonable way to try to do this would be simply to invalidate the `heights`
    // cache AND reuse all displayed views because the loop checks
    // pref height only for appearing views
    contentSize.width = bounds.width

    // Sanity check: every displayed view should intersect with `bounds`
    // — otherwise we haven't reused some rows
    if !isUpdating {
        //assert(displayedRows.values.allSatisfy { $0.frame.intersects(bounds.insetBy(dx: 0, dy: -5)) })
    }

    // TODO: assert that `bounds` are fully filled if not "minY < 0" or "maxY > lastRowFrame.maxY"

    // Sanity check: whatever we laid out is of known size
    // — otherwise we have an error in the loop logic
    assert(displayedRows.keys.allSatisfy { heights[$0] != nil })

    verticals.assertIndexSanity(against: dimensions)
    assertLayoutCorrectness(verticals: verticals, dimensions: dimensions)

    layoutListener?()
}
```