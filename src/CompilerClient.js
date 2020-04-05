import { Client, ClientOptions, Snowflake } from 'discord.js'
import CommandCollection from './commands/utils/CommandCollection'
import MessageRouter from './commands/utils/MessageRouter'
import { Compilers } from './utils/Wandbox'
import log from './log'

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

    this.commands = new CommandCollection(this);
    this.messagerouter = new MessageRouter(this, options);
  
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
    this.support_server = options.support_server;
    this.github_link = options.github_link;
    this.stats_link = options.stats_link;
    this.owner_id = options.owner_id;
  }

  setSupportServer(supportServer) {
    this.supportServer = supportServer;
  }

  async initialize() {
    try {
      await this.compilers.initialize();
    }
    catch (error) {
      this.emit('compilersFailure', error);
    }

    try {
      await this.messagerouter.blacklist.initialize();
    }
    catch(error) {
      log.error(`MessageRouter#Blacklist -> blacklist.json write failure (${error.message})`);
    }
  }
  /**
   * hook - Hooks bot processes to discord.js client
   */
  hook() {
    this.on('message', async (message) => {
      await this.messagerouter.route(message);
    });
  }
}
