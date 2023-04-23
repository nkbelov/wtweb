# A comprehensive guide to layout in UIKit

{#abstract} Discusses manual layout, the layout loop and auto layout.

It is early 2023 at the time of writing this. If you're reading this, already knowing that SwiftUI exists and maybe having written apps with SwiftUI, you might ask, why even care about UIKit, a technology that is seemingly becoming obsolete.

If the paragraph above convinced you — or you already came here with a good understanding of the shortcomings of SwiftUI — welcome. In this text, I will go through the majority of UIKit's APIs for view layout.

This is not a text for absolute beginners. If you're reading this, you should be comfortable with programming in general, and although I'm not referring to any advanced features particular to Swift or any other language. This is also not a tutorial on how to build anything particular. Instead, this is a text that aims to comprehensively discuss one particular set of `UIKit` APIs, also filling the gaps of its documentation.   

## Coordinate systems
Before talking about anything layout, we'll need to discuss coordinate systems.

Coordinate systems give geometrical or spatial meaning to the numbers that we use to describe sizes, positions, measurements and so on. A coordinate system specifies three things: the *origin*, the *unit* and the *directions* — the three components that you require to be able to tell where things are and how large they are. For instance, to precisely locate a building, you could say that "the building is 200 meters to the West from here" — whereby *here* is the origin, *meters* is the unit, and *West* is the direction.

UIKit uses two-dimensional cartesian coordinates with:
- the *top left corner* of the screen as the origin, the `(0,0)` point,
- the `x` coordinate increasing to the right and the `y` coordinate increasing *downward*,
- the unit being a `point`, the size of a single pixel on the original iPhone screen[^point].

[^point]: With the iPhone 4 and Retina displays, the pixel density first doubled, and later on became three times that of the original pixel density. The point size, though, *remains the same* — that is, something that is 10 points wide will look identical in the real world on any of those screens, but occupy 10, 20 or 30 pixels, respectively. Because in practice we rarely have to think about exact pixel measurements, you almost never have to think about it, and having the point size the same accross all screens makes sure that your UI has the same scale on all devices.

I want to stress again that in UIKit, *the origin is in the top left corner*, and the `y` coordinate grows moving down. This is somewhat unusual — most often you see the `y` coordinate increase upward with the origin on the bottom left — but this doesn't take a long time to get used to, and the rest of UIKit is built around this fact.

## Manual layout
### Rectangles and frames
Everything you see on the screen is a collection of rectangular regions of pixels laid on top of one another. These are what UI frameworks call *views*, and their arrangement on the screen is what ultimately creates the user interface. If you have ever worked with a graphical editor like Adobe Photoshop, the concept of layers there directly corresponds to views in a UI framework.

It is important to fix that views are *always* rectangular — this is because, ultimately, the contents of a view are normal computer images as we know them, and computer images are rectangular grids of pixels. Even if you see a circular button, it is still a rectangle that contains an *image* which is a circle; there are just transparent pixels near the corners that give it a rounded appearance.

What is a rectangle *for a computer*? While it's a very everyday concept, to be able to set layout programmatically, we need to know how rectangles are represented in the `UIKit`. Quite intuitively, a rectangle has a width and a height. These give us the *size*, or the *dimensions*, of the rectangle, and are the first part of the equation. The second part would be that rectangle's *position* or *location* on the screen. We find ourselves in a 2D coordinate space, so the position of an object would consist of an `x` and a `y` coordinate. Because a rectangle is a "voluminous" object however, it would be impossible to specify "the position" of the whole rectangle at once; rather, we can only speak of the positions of either of its corners or, perhaps, its center. The top left corner of a `CGRect` is what was chosen to be its "origin", analogous to the coordinate system itself, and whenever we lay a rectangle on screen, we ultimately set the position of that origin corner:

[TODO image of rect]

If one was to declare `CGRect` in Swift, it would look like this:

```swift

struct CGPoint {
    var x: CGFloat
    var y: CGFloat
}

struct CGSize {
    var width: CGFloat
    var height: CGFloat
}

struct CGRect {
    var origin: CGPoint
    var size: CGSize
}
```

These properties are enough to *uniquely* describe a rectangle — that is, given two `CGRect`s with identical `x`, `y`, `width` and `height`, they will perfectly overlap on the screen (and indeed, the expression `rect1 == rect2` returns `true` if and only if those four properties coincide). So, we can fix the following:

> A rectangle is described by the position of its top left corner via a `CGPoint` and its dimensions via a `CGSize`.

Notice that now it is possible to compute the coordinates of the other three corners of the rectangle: the `x` coordinate of the top right corner would be `origin.x + size.width`, for instance.

[TODO: Show "frame relations", i.e. that the third is computable from two]

This realisation is at the core of understanding layout. Whenever you want to position a view somewhere on the screen, you will have to make sure that you compute `origin.x`, `origin.y`, `size.width` and `size.height` for the view's `frame` *somehow*. You can initialize and set the whole `CGRect` directly:

```swift

myView.frame = CGRect(x: 0, y: 0, width: 50, height: 100)
```

You can set each property individually:

```swift

myView.frame.origin.x = 0
myView.frame.origin.y = 0
myView.frame.size.width = 50
myView.frame.size.height = 100
```

You can copy a different view's frame and then adjust it slightly:


```swift

myView.frame = otherView.frame
myView.frame.origin.x += 50
```

— there are endless possibilities that you can leverage depending on the use case.

### On-demand rendering
We have just discussed how to calculate layout for a view; the remaining questions are, *where* and *when* to do it. To do layouts reliably, as well as to make the process performant, we need to understand how the layout process works.

As you may know, the interactivity of UI-based applications is achieved by repeatedly updating the screen many times a second (60 or 120 frames per second on most devices — this number is also called the *framerate*) to achieve the feeling of motion and feedback — just like in a film. When done without additional consideration, the app would just re-render the whole screen for each of those frames. Giving that a usual app consists of hundreds of views, overlays, submenus and what not, this obviously would be tremendously inefficient, which is especially critical for mobile devices, as any unnecessary computation contributes to needlessly draining the battery charge. Usually indeed, after the user interacts with the app and the screen updates once, the contents (and hence the layout) stay the same until the next interaction — so the app can keep presenting the same contents for the following frames. Another observation is that more often than not, even if something changes on the screen, perhaps in response to a button tap, it's only a *small portion* of the screen — say, just a piece of text or the colour of a button — so there's no need to re-render *all* of the screen, just that particular portion.

Understanding these things is important for two reasons: on the one hand, they serve as the basis for the design of `UIKit` (and pretty much every UI framework in general); keeping these in mind will make the API design appear less cryptic. The second reason is that if you follow these principles yourself, you will be able to achieve much higher performance for your apps.

This — render only once and only the necessary minumum — is what we call *on-demand rendering*.

### The layout loop

To implement on-demand rendering, the framework would need to follow the following logic:

1. have a way to check, or *poll* the app state to see if there are any views that need a re-render,
2. when it's time to render the screen, look at all the views that decided that they need a re-render,
3. render them, then cache.

Simplifying greatly, this is how this process could look in Swift pseudocode:
```swift

while appIsRunning {
    if timeToPresentNextFrame {
        for view in viewsInTheApp {
            if view.somethingHasChanged {
                view.refresh()
            }
        }
    }
}
```

The outermost (`while`) loop would be the layout loop — i

Looking at the code, we see that every view has to have two features to support this logic: firstly, it has to be able to tell if something has changed that necessitates an update, and secondly, there should be a function that the frameworks calls 



### Meet `layoutSubviews()`

Every `UIView` subclass has a method named `layoutSubviews()` which is *the* method that gets called from within the layout loop. If you read the documentation, it says

> You should not call this method directly. If you want to force a layout update, call the setNeedsLayout() method instead to do so prior to the next drawing update.

This is precisely the behaviour that 



## Auto Layout
We will now discuss Auto Layout, by far the most widespread way to create layouts in UIKit. Auto Layout has numerous convenience advantages over the manual layout, but it must be noted that, in the end, it too ends up calculating the `frame`s of the `UIView`s you're working with. Also, while a lot of sources discuss how great and powerful it is, oftentimes people don't mention the situations where Auto Layout simply doesn't work at all or is so inconvenient that manual layout would be a much better choice — typography would be one of the most notorious cases. Additionally, Auto Layout almost by definition has a higher performance cost, and while in the absolute majority of cases this cost is negligible, knowing when to switch to a manual layout algorithm can dramatically improve an app's performance.

Auto Layout (or rather, the engine behind it) is what's known as a *constraint solver*. It is actually a very prominent class of algorithms all throughout mathematics and computer science, and while we won't dig very deep into the maths aspects of it, understanding some terminology and concepts is quite crucial to be able to decipher the error messages that sometimes pop up, as well as to generally understand how to manipulate layouts properly.

On an abstract level, constraint solvers, as the name suggests, work with *constraints* in an attempt to produce a set of values that leaves all the constraints satisfied. Practically speaking, in view layout, your constraints will be something like "the width of a button should be smaller than the width of the image above it" or "the username label should be positioned to the right of the profile picture". These are constraints in the sense that they constrain, or restrict, the possible position and/or size of a view.

Initially, all views are unconstrained, that is, they could theoretically be placed anywhere on the screen[^unconstrained], and even outside of its bounds. As you add more constraints to your views, their position on the screen, as well as the size, become more and more defined. Your goal, when working with auto layout, is to specify enough constraints that the `frame` of the view becomes fully defined, i.e. the engine knows exactly where to position your view and what size it should be. 

[TODO: Show constraints view]

[^unconstrained]: In practice, an unconstrained view will just assume the default frame, that is, a `CGRect.zero` — positioned at the top left corner of its superview with zero height and width.

Being more mathematical now, a constraint is a restriction on the range of some value. An unconstrained value can be any number[^finite]. With any additional constraints, the permissible range for the value shrinks — a "less than" constraint will shrink the permissible range from above down to the constraint value, and a "greater than" constraint will shrink the permissible range from below. An equality constraint collapses the permissible range down to just one possible number.

[^finite]: To be more pedantic, any *finite* number — that is, any float that is not `NaN` or plus or minus `Infinity`.

The most crucial property of constraints is that every single of them constraints has to be satisfied *simultaneously*. In other words, while each constraint defines a permissible region for a value, a combination of constraints results in an intersection of those ranges. In yet other words, they act as an AND expression: a value satisfies its constraints when it satisfies its first constraint *and* its second constraint *and* (its third constraint, if it exists — and so on).

The first important observation is that a less-than and greater-than constraint combined can be equivalent to an equality constraint. For instance, whenever you say that a width has to be less than 50 points and greater than 50 points *at the same time*, the only value it can be is then exactly 50 points. That is, if you write

[TODO: Show intersection]

The second observation is that inequality constraints can be *redundant*. For instance, let's say you constrain the height of a view to be smaller than 125 points, and then constrain again to be smaller than 100 points. The net result is that 

[TODO: Show intersection]

For all views, there are also two intrinsic constraints that are sometimes easily forgotten, and it is that *the width and the height are always non-negative*. That is, both the width and the height have a greater than 0 constraint by default, and this can't be disabled.

Let's set up a view with some constraints to see how this works, and 
[TODO: Show code, set up some constraints]

Now that we set up some basic constraints, let's discuss some additional implications of this system.