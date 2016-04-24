TrelloBST
=========

A tool to make Travis-ci and appveyor .yml files that push build statuses to a trello board.
Generates a generic configuration which needs to be modified according to your build.
Also attaches link to the build artifacts. Directory defined in the BUILD_DIRECTORY environment variable will be compressed and linked in the build status.
Includes a direct link to the build log.
Status message modifiable, easier way to do so coming in version 2.0 (code in the .yml doing the actual build status push is a bit of a mess I admit).


Build Status
------------

* Travis CI OSX/Linux: ![](https://travis-ci.org/Cyberunner23/TrelloBST.svg?branch=master)
* AppVeyor CI Windows: ![](https://ci.appveyor.com/api/projects/status/pticmhbvy4unm0uj?svg=true)
