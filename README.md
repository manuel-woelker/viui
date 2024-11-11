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