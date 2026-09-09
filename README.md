# Zopra

A reactive framework for building highly reactive apps.

                    #[component]
                         │
                         ▼
              ┌─────────────────────┐
              │ ComponentOwner      │
              │                     │
              │  Reactive Graph     │
              │  Signals            │
              │  Memos              │
              │  Dynamic Nodes      │
              └──────────┬──────────┘
                         │
                    setup once
                         │
          ┌──────────────┴──────────────┐
          ▼                             ▼
    signal(initial)                  view! { ... }
          │                             │
          ▼                             ▼
    SignalId + handles          Static GPUI tree
                                      │
                                      ├── div
                                      ├── button
                                      └── DynamicElement
                                             │
                                             ▼
                                      Reactive NodeId

# How does it work?

This is purely a Solid style framework for building highly reactive apps.
In order for us to build something of this magnitude, we need to understand how it works.

If we look at this from a top level perspective, it kinda does one thing, and it does it pretty well:

It takes:

```rust
let (count, set_count) = signal(0);
let doubled = memo(|count.get() * 2);
```

```rust
DynamicElement::new(
    owner,
    doubled_node,
    |value, window, cx| {
        div().child(format!("Double: {}", value))
    }
)
```

into

UI:

```
Double: 0
set_count(5) -> count = 5
Now the double should be 10
```

So, how do we achieve this?
In this specific case, we can see that doubled depends on count.
This concept is called _Reactivity_.
That's exactly what we'll solve using zopra

### So, how do we do this?

Take an example:

```
A = 10
B = A × 2
```

Here, B depends on A.
So, if A changes, B should automatically update.
If A changes to 20, B should automatically update to 40.
So how do we know that B depends on A?
We make a graph of dependencies, called the dependency graph.
It kinda looks like this:

```
A -> B
```

If A changes, B should automatically update.

But before we go there, I want you to have a good understanding of the 4 concepts:

- Signal
- Node
- Scope
- Owner

### Signal

This is a signal:

```rust
let (count, set_count) = signal(0);
```

When you call the signal function, it returns 2 methods, `get` and `set`.
One is used to get the value, and the other is used to set the value.
What does a signal actually look like tho?

Signal:
value
subscribers

value is the current value of the signal.
subscribers is a list of functions that are subscribed to this signal.
We are using a design pattern called The [Observer Pattern](https://refactoring.guru/design-patterns/observer) here.
Every time the value of the signal changes, all the subscribers are notified to update their values.
Remember the previous dependency graph, that's exactly what we'll do here

### Node

This is literally a function. A function whose result we want to reactively update.
(Or sometimes cache too!)

In this case:

```rust
let doubled = memo(|| count.get() * 2);
```

the `doubled` node will automatically update whenever `count` changes.

A Node roughly looks like this:

```
NODE:
function: count * 2
cached_value: 20
dependencies: [count]
subscribers: []
```

### Scope

This is similar to the `Scope` in any other language. It is a way to group nodes together. But more importantly, it answers one thing: Which things should die together? Although it sounds romantic, it is actually a very practical concept.

```bash
Component
   │
   ├── signal A
   ├── signal B
   └── memo
```

Since signal A,B and memo all are on the same scope as Component, it drops them when the Component is dropped

A Scope looks like this:

```rust
pub struct ScopeData {
    parent,
    children,
    signals,
    nodes,
    cleanups,
}
```

### Owner

This is the single most important concept in Zopra.
One component gets one owner.

The owner has all the information about the component.
An owner looks like this:

```bash
ComponentOwner
│
├── Signals
├── Nodes
├── Scopes
│
├── current scope
├── current computation
│
└── GPUI notifier
```

Okay, since you've understood the 4 basic concepts, let's take a look at the data-structure that Zopra uses: `Generational Arena`

## Arena?

The arena is a data-structure that Zopra uses to store nodes and scopes.
Instead of storing directly as objects, the runtime stores than as a giant array:
Each slot has a signal.

```bash
Signal Arena

index
  0      Signal A
  1      Signal B
  2      Signal C
  3      empty
  4      Signal D
```

And instead of passing around the actual objects, we pass `Id`.
And `Id` is roughly this:

```rust
Id {
    index: 2,
    generation: 0
}
```

Here, `index` is the `index` into the arena, and `generation` is used to detect when the arena needs to be resized.

This is to avoid the ABA problem.
Let's take an example:

```bash
0 Signal A
```

You delete signal A, and then signal B is moved to index 0.

```bash
0 Signal B
```

Now, old code and code that's using the old `Id` can accidentally access Signal B.
So, we add a fencing token (generation) to the `Id`.

So, if we re-run the same scenario right now, it roughly looks like this:

```bash
Signal A
index = 5
generation = 0
```

Deleting Signal A will increment the generation to 1.

```bash
slot 5
generation = 1
```

Now signal B is moved to index 5, but the generation remains 1.

```bash
slot 0
generation = 1
```

Old ID: `Id { index: 0, generation: 0 }`
New ID: `Id { index: 5, generation: 1 }`

An Arena looks like this:

```rust
pub struct Arena<T> {
    entries: Vec<ArenaEntry<T>>,
    free: Vec<u32>,
}
```

On insertions (`arena.insert(value)`), the arena checks if there's an available slot in `free`. If there is, it uses that slot. Else, it creates a new slot.

This gives us `O(1)` insertions, `O(1)` lookups, and `O(1)` deletions.

Okay, since the basics are covered, let's see how the get funciton works:

```rust
pub fn get(&self, id: Id) -> Option<&T>
```

This means that the Id has:

```bash
index: 0
generation: 1
```

it then looks at the entry at index 0 and checks if the generation matches. If it does, it returns the Object. If not, it returns `None`.
This is made to ensure stale IDs don't accidentally access new objects.
