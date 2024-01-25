# Mediacenter

This is a collection of utilities to be run on a computer conntected to a TV.

## Motivation

I would like to be able to drive operate the computer connected to the TV using a remote controller and use a web browser to consume the content. Solutions like [Kodi](https://kodi.tv/) are great for self hosted content, but (at least from my experience) they are bloated and extermely fragile when it comes to plugins that connect to third parties: the plugins are very brittle, not well maintained and the whole thing might crash. I personally find that using a web browser is a more effective way of consuming content. In addition to that, in practice, the vast majority (95%+) of interaction can be performed with a remote controller and for the few times when additional input is requiered, it is fine to use a keyboard or an alternative input method.

This is a minimal product that provides just enough functionality to satisfy a very niche use case.

## Prerequisites

The TV must be a CEC enabled device.

Linux with a web broswer.

## Installation

Clone the repository.

The `cec-mouse` script and the `frontend` web app must be started at startup. Additionally, a web browser should be opened to point to the address of the frontend app.