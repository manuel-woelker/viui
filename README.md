viui is an experimental UI framework with a focus on feedback, flexibility and efficiency

## Goals

- Build a UI framework that is a joy to use
- Make it easy to quickly iterate on UIs
- Make it easy to write meaningful tests for UIs

## Vision

**Fast feedback**\
Tight feedback loops are a force multiplier for any UI work. Iterating quickly on an idea let's us more easily get to a
better solution. Concretely this means the UI can be modified while the application is running.

**Flexibility**\
The UI should not be tied to any specific implementation but rather sit on top of an abstraction layer that accepts
drawing commands and emits user events

**Efficiency**\
The UI should reduce unnecessary work, both developer and machine work. Specifically this means only drawing
things that have changed.

## Implementation ideas

**Hot reloading, declarative UI**\
The UI is written as simple files in a custom UI modelling language. Changing this definition updates the UI while it is
runnning. A special design mode should allow editing the UI while it is running.

**Render tests**\
Since UIs produce a list of render commands, render tests can be written on render commands or as SVG screenshots.

**Integrated persistent undo**\
Using the fine-grained change detection mechanism allows keep track of and reverting to previous states even across
application restarts.

## Separation of concerns

Concerns should be decoupled from each other, specifically:

### State & logic separation

** Application Data model**\
The application data should exist independently of any UI.

** Application State Transition Logic**\
The logic for transitioning from one application state to another should be independent of the UI, and be able to run
without any UI.

### Presentation separation

** UI Layout
The positioning of elements in the UI should be independent of its look and content.

** Styling
The styling of elements (e.g. colors, font faces, margins, borders, etc.) in the UI should be independent of its layout

** Text
The text content should be independent as well, to allow easier translation

### UI loop separation

** Render commands
The actual drawing is decoupled by emitting a list of render commands, which are then used to actually draw the UI.

** UI painting
The UI is drawn in a separate thread, based on the abstract render commands.

