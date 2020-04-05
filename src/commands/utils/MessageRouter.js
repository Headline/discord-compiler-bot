import CompilerClient from '../../CompilerClient'
import {Message} from 'discord.js'
import CompilerCommandMessage from './CompilerCommandMessage'
export default class MessageRouter{

  /**
   * Handles command routing argument parsing
   * @param {CompilerClient} client
   * @param {RouterOptions} options
   */

   constructor(client, options) {
    /**
     * Discord client
     * @type {CompilerClient}
     */
    this.client = client;

    /**
     * Command prefix
     * @type {string}
     */
    this.prefix = options.prefix;
  }


  /**
   * route - Routes a message to an appropriate command handler
   *
   * @param  {Message} message discord.js message
   * @return {boolean} true on successful routing
   */
  async route(message) {
    if (!message.content.startsWith(this.prefix))
      return false;

    const commandStr = message.content.substr(this.prefix.length).match(/(?:[^\s"]+|"[^"]*")+/g);
    if (!commandStr)
      return false;

    const commandFunc = this.client.commands.find(f => f.name == commandStr[0]);
    if (!commandFunc)
      return false;

    this.client.emit('commandExecuted', commandFunc)

    await commandFunc.run(new CompilerCommandMessage(message));
  }
}
