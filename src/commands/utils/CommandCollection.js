import { join } from 'path';
import { readdir } from 'fs';
import { Collection } from "discord.js";
import CompilerClient from "../../CompilerClient";
import CompilerCommand from './CompilerCommand';

export default class CommandCollection extends Collection {
    /**
     * Creates a new store for commands
     *
     * @param {CompilerClient} client
     */
    constructor(client) {
        super();

        /**
         * CompilerClient
         *
         * @type {CompilerClient}
         */
        this.client = client;
    }

    /**
     * Register a command
     *
     * @param {CompilerCommand | Function} command
     */
    registerCommand(command) {
        if (typeof command === 'function') {
            command = new command(this.client);
        }

        if(!(command instanceof CompilerCommand)) {
            throw new Error('Invalid command class: ' + command);
        }

        if (this.some(cmd => cmd.name === command.name)) {
            throw new Error(`Command name "${command.name}" is already registered`);
        }

        this.set(command.name.toLowerCase(), command);

        /**
         * Emitted when a command is registered
         *
         * @event CompilerClient#commandRegistered
         * @param {CompilerCommand} command Command that was registered
         */
        this.client.emit('commandRegistered', command);
    }

    /**
     * Register commands within a directory
     *
     * @param {string} path
     */
    async registerCommandsIn(path) {
        readdir(path, (err, files) => {
            if (err) {
                throw new Error(err);
            }

            const commandFiles = files.filter(f => f.split('.').pop() === 'js');

            if (commandFiles.length == 0)
                return;

            let cPath, req;

            for (let commandFile of commandFiles) {
                cPath = join(path, commandFile);

                req = require(cPath);

                if (typeof req.default === 'function') {
                    this.registerCommand(req.default);
                }
            }
        })
    }
}
