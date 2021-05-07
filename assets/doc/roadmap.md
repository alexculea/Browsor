# Roadmap
## Coming next
- Ensure window opens with its center where the mouse cursor is
- Add CLI argument `--register` for integrating with the OS as a web browser capable program, also add `--unregister` for uninstalling/clean up
- Support multiple Firefox profiles, show an entry for each Firefox profile found and indicate which profile is being opened
- Design an app icon


## Mid term
- Allow manual drag and drop for reordering of the list
- Cursor capturing (exclusive mode)
  - Option that can be turned on by the user (off by default) via a checkbox at the bottom of the list
  - When on, hides the system mouse cursor as soon as the selection window appears
  - Auto focuses the selection window
  - The window responds to the scroll wheel by moving up and down the selected item
  - Left click activates the selected item
  - ESC key closes the window
- Show window imediately regardless if the system browsers (and their icons) have been loaded, use a loading spinner in place of the list
  - Use a background thread to discover what browsers are available on the system and to load their associated app icons

## Far future
- Add support for Linux and MacOS
- Add learning algorithm that predicts the choice made by looking at: time of the day, location of the device, program from where the link was clicked


# Governing User story
The vision of the utility

- As a user I start the program and am guided to set it as my default OS browser, replacing my existing choice
- Once I finished the guided setup, I then:
  - click a link anywhere in my system such as a 3rd party app and instead of a browser opening I see the program showing a list of browsers I choose from
  - select a browser from the list to open my URL
    - I should be able to use either my keyboard arrows or the mouse scroll wheel to select a browser
    - and I don't have to manually focus the window
  - but I will always see the window on top of anything else
  - and I can get rid of the window if I either: hit ESC, click somewhere else
- It'd be very nice if I'd:
  - benefit from a performant operation taking at most half a second for the window regardless of my system config
  - see list ordered by how often I use a certain browser and what are my usage patterns
    - maybe the utility could predict what is the browser I'm about to open by looking at my usage patterns
  - be inspired by the beatufilul design and UX of the app
    - potentially having an active, calming, animated background in the main selector window