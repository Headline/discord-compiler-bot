# Structure

```
src/                        #  Our source folder
├── build.rs                #  Build script to embed git hash for ;botinfo commmand
|
├── main.rs                 #  Code entry point, command registration, and client spawning
│
├── cache.rs                #  Sets up the cache to be used for the bot's resources
│
├── events.rs               #  All discord event handlers excluding command callbacks
│
├── apis/                   #  The home of any involved API integration
│   └── dbl.rs              ## Discord bot's list webhook logic
│
├── commands/               #  Module containing all of our command logic
│   └── ...
│
├── stats/                  #  Module containing all statistics tracking logic
│   ├── stats.rs            ## StatsManager abstraction for common code paths
│   └── structures.rs       ## Stats request models & request dispatch
│
└── utls/                   #  Module with random utilities to be used throughout the project
    ├── constants.rs        ## Constants
    ├── discordhelpers.rs   ## Embed builders, menu builders, general tools to be used
    └── parser.rs           ## Compile/Asm command parsing logic
    ```
