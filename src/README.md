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
├── apis/                   #  The home of any involved API integration
│   ├── dbl.rs              ## Discord bot's list webhook logic
│   ├── wandbox.rs          ## Primary function to execute wandbox requests 
│   └── godbolt.rs          ## Same as above, but for godbolt requests 
│
├── commands/               #  Module containing all of our command's logic
│   └── ...
│
├── stats/                  #  Module containing all statistics tracking logic
│   ├── stats.rs            ## StatsManager abstraction for common code paths
│   └── structures.rs       ## Stats request models & request dispatch
│
└── utls/                   # Module with random utilities to be used throughout the project
    ├── discordhelpers/     # Module with some discord shortcuts to help keep the project clean
    │   ├── mod.rs          ## Menu handlers & other commonly used functions
    │   └── embeds.rs       ## Tools that builds our outputs & prepares them for display
    ├── constants.rs        ## Constants
    └── parser.rs           ## Compile/Asm command parsing logic
```
