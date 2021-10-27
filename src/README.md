# Structure
Here's a little breakdown of the bot's source structure. Hopefully this can help you get your bearings on where to find what you're looking for.
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
├── commands/               #  Module containing all of our command's logic
│   └── ...
│
├── managers/               #  Module containing all statistics tracking logic
│   ├── compilation.rs      ## StatsManager abstraction for common code paths
│   └── stats.rs            ## Manager used to handle all interactons with stats/tracking
│
├── stats/                  #  Module containing all statistics tracking structures
│   └── structures.rs       ## Stats request models & request dispatch
│
├── apis/                   #  The home of any involved API integration
│   └──dbl.rs               ## top.gg's webhook logic
│
└── utls/                   # Module with random utilities to be used throughout the project
    ├── discordhelpers/     # Module with some discord shortcuts to help keep the project clean
    │   ├── mod.rs          ## Menu handlers & other commonly used functions
    │   └── embeds.rs       ## Tools that builds our outputs & prepares them for display
    ├── blocklist.rs        ## Our blocklisting strategy to preven abuse
    ├── constants.rs        ## Constants
    └── parser.rs           ## Compile/Asm command parsing logic
```
