# Proud force-unwrapping

Force-unwrapped optionals are often frowned upon. They don't have to be.

Here's a list of cases where force-unwrapping is provable to trigger no crashes at runtime and thus safe to use:

### The `nil` case has been exhausted

```swift
if dictionary[key] == nil {
    // ...
} else {
    let value = dictionary[key]!
}
```

### A non-`nil` value has just been assigned

```swift
if optional == nil {
    optional = "Hello!"
}

let string = optional!
```

### After an assertion or precondition
```swift
assert(optional != nil)
let val = optional!.property
```

### It is an algorithmic invariant
```swift
let names = ["Kate", "John", "Lisa"]
let dict = Dictionary(uniqueKeysWithValues: zip(1...3, names))
let lisa = dict[3]!
```

```swift
if !array.isEmpty {
    let firstItem = array.first!
    let lastItem = array.last!
}
```

### The value can't be `nil` for a well-formed input
```swift
let url = URL("https://www.google.com/")!
```

```swift
let double = Double("0.01")!
```
---

*May you force-unwrap with pride.*