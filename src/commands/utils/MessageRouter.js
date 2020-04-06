import log from '../../log'
import fs from 'fs'
import { Message } from 'discord.js'

import CompilerClient from '../../CompilerClient'
import CompilerCommandMessage from './CompilerCommandMessage'

export default class MessageRouter {

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

    /**
     * Wrapper which allows for easy guild blacklsting
     * @type {GuildBlacklist}
     */
    this.blacklist = new GuildBlacklist();
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

    // Message from discord system (lul?)
    if (message.system)
      return false;

    // Disable direct message 
    // Allow this someday maybe?
    if (message.guild == null)
      return false;


    if (this.blacklist.isBlacklisted(message.guild.id)) {
      const msg = new CompilerCommandMessage(message);
      await msg.replyFail('This guild has been blacklisted from executing commands.'
        + '\nThis may have happened due to abuse, spam, or other reasons.'
        + '\nIf you feel that this has been done in error, request an unban in the support server.');
      return;
    }

    const commandStr = message.content.substr(this.prefix.length).match(/(?:[^\s"]+|"[^"]*")+/g);
    if (!commandStr)
      return false;

    const commandFunc = this.client.commands.find(f => f.name == commandStr[0]);
    if (!commandFunc)
      return false;

    if (commandFunc.developerOnly && message.author.id != this.client.owner_id)
      return false;

    /**
     * Event that's called before every command execution from a client
     * 
     * @event CompilerClient#commandExecuted
     */
    this.client.emit('commandExecuted', commandFunc)

    await commandFunc.run(new CompilerCommandMessage(message));
  }
}

/**
 * Helper class that wraps a file to provide basic blacklisting
 */
class GuildBlacklist {
  /**
   * Builds a guild blacklist
   */
  constructor() {
    this.data = {
      guilds: [],
    }
  }

  /**
   * Determines whether or not a guild is blacklisted
   * 
   * @param {string} guildid 
   * @return {boolean} true if blacklisted
   */
  isBlacklisted(guildid) {
    return this.data.guilds.includes(guildid);
  }

  /**
   * Blacklists a guild
   * 
   * @param {string} guildid 
   * @return {Promise} 
   */
  async blacklist(guildid) {
    this.data.guilds.push(guildid);
    await this.writeFile();
  }

  /**
   * Blacklists a guild
   * 
   * @param {string} guildid 
   * @return {Promise} 
   */
  async unblacklist(guildid) {
    this.data.guilds.splice(this.data.guilds.indexOf(guildid), 1);
    await this.writeFile();
  }

  /**
   * Loads blacklist from file. Creates blacklist.json if not found
   * 
   * @return {Promise}
   */
  async initialize() {
    try {
      let data = await this.readFile();
      this.data = JSON.parse(data);
    }
    catch (error) {
      log.warn('MessageRouter#Blacklist -> blacklist.json not found, creating...');
      try {
        await this.writeFile();
      }
      catch (error) {
        throw (error);
      }
    }
  }

  /**
   * Writes cached blacklist to blacklist.json
   * 
   * @return {Promise}
   */
  async writeFile() {
    return new Promise((resolve, reject) => {
      fs.writeFile('blacklist.json', JSON.stringify(this.data), 'utf8', (err) => {
        if (err) {
          reject(err);
        }
        resolve()
      });
    });
  }

  /**
   * Reads cached blacklist from blacklist.json
   * 
   * @return {Promise}
   */
  async readFile() {
    return new Promise((resolve, reject) => {
      fs.readFile('blacklist.json', 'utf8', function (err, data) {
        if (err) {
          reject(err);
        }
        resolve(data);
      });
    });
  }
} 