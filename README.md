# Browser Selector
Choose the browser you will run any time a link is opened from a click or any other action.


![]( assets/program-screenshot.png )

# End goal story of this utility
Not all is implemented and it might change.
- As a user I:
  - start the program and be guided to set it as default OS browser
  - click a link anywhere in my system such as a 3rd party app and instead of a browser opening I see the program showing a list of browsers 
  - select browsers from the list by using my keyboard arrows or the mouse scroll wheel
  - don't have to manually focus the window
  - will always see the window on top of anything else
  - can get rid of the window if I either: hit ESC, click somewhere else
  - only the use program on Windows
  - benefit from a performant operation taking at most 300msec for the window to come up on screen when I use an SSD as my system drive
  - see list ordered by how often I use a certain browser


# Roadmap
## Short term
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
