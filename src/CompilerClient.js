import { Client } from 'discord.js'

import CommandCollection from './commands/utils/CommandCollection'
import MessageRouter from './commands/utils/MessageRouter'
import { Compilers } from './utils/Wandbox'
import { SupportServer } from './SupportServer'
import { StatisticsAPI } from './StatisticsTracking'
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
    this.stats = null;

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
     * Support server helper tools
     * @type {SupportServer}
     */
    this.supportServer = null;

    /**
     * Setup compilers cache
     */
    this.compilers = new Compilers(this);

    /**
     * Environment Variables
     */
    this.loading_emote = options.loading_emote;
    this.prefix = options.prefix;
    this.invite_link = options.invite_link;
    this.discordbots_link = options.discordbots_link;
    this.join_log = options.join_log;
    this.compile_log = options.compile_log;
    this.dbl_log = options.dbl_log;
    this.github_link = options.github_link;
    this.stats_link = options.stats_link;
    this.owner_id = options.owner_id;
  }

  /**
   * Sets the support server property
   * 
   * @param {SupportServer} supportServer 
   */
  setSupportServer(supportServer) {
    this.supportServer = supportServer;
  }

  /**
   * Sets the statistics api
   * @param {StatisticsAPI} stats 
   */
  setStatsAPI(stats) {
    this.stats = stats;
  }
  /**
   * Initializes compiler client's resources
   */
  async initialize() {
    try {
      await this.compilers.initialize();
    }
    catch (error) {
      /**
       * Event that's called when the compilers were unable to initialize
       * 
       * @event CompilerClient#compilersFailure
       * @type {Error}
       */
      this.emit('compilersFailure', error);
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
    this.on('message', async (message) => {
      await this.messagerouter.route(message);
    });
  }
}
