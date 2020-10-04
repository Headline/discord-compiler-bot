import { GuildEmoji, Client, Guild } from 'discord.js'

import CommandCollection from './commands/utils/CommandCollection'
import MessageRouter from './commands/utils/MessageRouter'
import { Wandbox } from './utils/apis/Wandbox'
import { StatisticsAPI } from './utils/apis/StatisticsTracking'
import { Godbolt } from './utils/apis/Godbolt'
import { CompilationFixer } from './utils/CompilationFixer'
import log from './log'
import MemorySweeper from './utils/MemorySweeper'

/**
 * discord.js client with added utility for general bot operations
 */
export default class CompilerClient extends Client {

  /**
   * constructor - Creates a Compiler bot client
   *
   * @param  {CompilerClientOptions} options = {} options for bot creation (prefix, etc)
   */
  constructor(options = {}) {
    super(options);

    /**
     * Statistics tracking API
     * @type {StatisticsAPI}
     */
    this.stats = new StatisticsAPI(this, options.stats_api_link);

    /**
     * Collection of commands for lookup
     * @type {CommandCollection}
     */
    this.commands = new CommandCollection(this);

    /**
     * Handles command routing, owner checks, and arg splitting
     * @type {MessageRouter}
     */
    this.messagerouter = new MessageRouter(this, options);

    /**
     * Setup compilers cache
     * @type {Wandbox}
     */
    this.wandbox = new Wandbox(this);

    /**
     * Setup godbolt cache
     * @type {Godbolt}
     */
    this.godbolt = new Godbolt(this);

    /**
     * Setup automated code fixer
     * @type {CompilationFixer}
     */
    this.fixer = new CompilationFixer();

    /**
     * Sweeps discord.js caches & deallocates memory
     * @type {MemorySweeper}
     */
    this.sweeper = new MemorySweeper(this, 60);

    /**
     * Determines whether the bot is in maintenance mode
     * @type {boolean}
     */
    this.maintenance = options.maintenance;

    /**
     * Environment Variables
     */
    this.loading_emote = options.loading_emote;
    this.finished_emote = options.finished_emote;
    this.prefix = options.prefix;
    this.invite_link = options.invite_link;
    this.discordbots_link = options.discordbots_link;
    this.join_log = options.join_log;
    this.compile_log = options.compile_log;
    this.dbl_log = options.dbl_log;
    this.github_link = options.github_link;
    this.stats_link = options.stats_link;
    this.owner_id = options.owner_id;
    this.stats_api_link = options.stats_api_link;
  }

  /**
   * Updates the presence with the updated server count
   */
  async updatePresence() {
    try {
      const count = await this.getTrueServerCount();
      if (this.maintenance)
        this.user.setPresence({activity: {name: `MAINTENENCE MODE`}, status: 'dnd'});
      else
        this.user.setPresence({activity: {name: `in ${count} servers | ${this.prefix}invite`}, status: 'online'});  
    }
    catch (e) {
      log.warn(`CompilerClient#updatePresence -> ${e}`)
    }
  }

  /**
   * Queries all shards for guild count & returns the sum
   * 
   * @return {Promise<number>}
   */
  async getTrueServerCount() {
    let values = await this.shard.fetchClientValues('guilds.cache.size')
    let guildCount = values.reduce((a, b) => a + b);
    return guildCount;  
  }

  /**
   * Finds an emoji and copies it's object
   * 
   * @param {string} snowflake emoji snowflake
   */
  findEmoji(snowflake) {
    const temp = this.emojis.cache.get(snowflake);
    if (!temp) return false;

    const emoji = Object.assign({}, temp);
    if (emoji.guild) emoji.guild = emoji.guild.id;
    emoji.require_colons = emoji.requiresColons;

    return emoji; 
  }
  /**
   * Returns the emoji object from all shards
   * @param {string} snowflake emoji id
   */
  async getEmojiFromShard(snowflake) {
    const emojis = await this.shard.broadcastEval(`(${this.findEmoji}).call(this, '${snowflake}')`);
    const emoji = emojis.find(emoji => emoji);

    const raw = await this.api.guilds(emoji.guild).get();
    const guild = new Guild(this, raw);
    return new GuildEmoji(this, emoji, guild);
  }

  /**
   * Pushes the server count to the custom stats api
   * 
   * @param {number} guildCount number of guilds
   */
  updateServerCount(guildCount) {
    if (this.shouldTrackStats())
      this.stats.insertServerCount(guildCount);
  }

  /**
   * Determines if statistics should be tracked
   * 
   * @returns {boolean}
   */
  shouldTrackStats() {
    return (this.maitenance)?false:this.stats_api_link;
  }

  /**
   * Initializes compiler client's resources
   */
  async initialize() {
    try {
      await this.wandbox.initialize();
    }
    catch (error) {
      /**
       * Event that's called when the compilers were unable to initialize
       * 
       * @event CompilerClient#wandboxFailure
       * @type {Error}
       */
      this.emit('wandboxFailure', error);
    }

    try {
      await this.godbolt.initialize();
    }
    catch (error) {
      /**
       * Event that's called when godbolt is unable to initialize
       * 
       * @event CompilerClient#godboltFailure
       * @type {Error}
       */
      this.emit('godboltFailure', error);
    }

    try {
      await this.messagerouter.blacklist.initialize();
    }
    catch (error) {
      /**
       * Event thats called when the blacklist is unable to be initialized
       * 
       * @event CompilerClient#blacklistFailure
       * @type {Error}
       */
      this.emit('blacklistFailure', error);
    }
  }

  /**
   * hook - Hooks command routing to discord.js client
   */
  hook() {
    this.on('message', (message) => {
      this.messagerouter.route(message);
    })
    .on('commandExecuted', (f) => {
      if (this.shouldTrackStats() && !f.developerOnly)
      {
        this.stats.commandExecuted(f.name);
        this.stats.incrementRequestCount();	
      }
    });
  }
}
