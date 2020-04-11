import CompilerClient from "../../CompilerClient";
import Message from "discord.js";
import "regenerator-runtime/runtime.js";

export default class CompilerCommand {

    /**
     * @typedef {Object} CommandInfo
     * @property {string} name - Name of the command
     * @property {string} description - Description of the command
     * @property {string[]} aliases - Aliases of the command
     * @property {boolean} [developerOnly = false] - Allow only developer use this command
     */

    /**
     * Create a new base command object
     *
     * @param {CompilerClient} client - Client object of Compiler
     * @param {CommandInfo} info - Command information
     */
    constructor(client, info) {
      /**
       * CompilerClient
       *
       * @type {CompilerClient}
       */
      this.client = client;

      /**
       * Command name
       *
       * @type {string}
       */
      this.name = info.name.toLowerCase();

      /**
       * Command description
       *
       * @type {string}
       */
      this.description = info.description || '';

      /**
       * Developer flag for non-public commands
       *
       * @type {string}
       */
      this.developerOnly = info.developerOnly || false;
    }

    /**
     * Abstraction interface for commands
     *
     * @abstract
     *
     * @param {Message} message
     */
    async run(message) {
      throw new Error(`${this.constructor.name} doesn't have a run() method.`);
    }

    /**
     * Abstraction for help description
     *
     * @abstract
     *
     * @param {Message} message
     */
    async help(message) {
      throw new Error(`${this.constructor.name} doesn't have a help() method.`);
    }

    /**
     * Format command  (prefix + command name)
     *
     * @return {string}
     */
    toString() {
      return this.client.prefix + this.name.toLowerCase();
    }
}
