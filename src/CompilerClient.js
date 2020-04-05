import {Client, ClientOptions, Snowflake} from 'discord.js'
import CommandCollection from './commands/utils/CommandCollection'
import MessageRouter from './commands/utils/MessageRouter'
import {Compilers} from './utils/Wandbox'

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

    this.compilers = new Compilers(this);

    this.loading_emote = options.loading_emote;
    this.prefix = options.prefix;
    this.invite_link = options.invite_link;
    this.discordbots_link = options.discordbots_link;
  }

  async initializeCompilers() {
    try {
      await this.compilers.initialize();
    }
    catch (error) {
      this.emit('compilersFailure', error);
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
