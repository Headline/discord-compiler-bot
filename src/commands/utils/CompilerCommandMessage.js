import { Message, StringResolvable, MessageOptions, MessageEmbed, Attachment, Author } from "discord.js";
import CompilerCommand from './CompilerCommand'

export default class CompilerCommandMessage {

    /**
     * Constructs a compiler message
     *
     * @param {Message} message
     */
    constructor(message) {
        /**
         * Underlying DJS message
         *
         * @type {Message}
         */
        this.message = message;

        /**
         * Command that this message triggers
         *
         * @type {CompilerCommand}
         */
        this.command = null;
    }

    /**
     * Creates a failure embed and sends it
     * @param {StringResolvable} message 
     */
    async replyFail(message) {
        const embed = new MessageEmbed()
        .setTitle('Critical error:')
        .setDescription(message)
        .setThumbnail('https://imgur.com/LxxYrFj.png')
        .setFooter("Requested by: " + this.message.author.tag)
        .setColor(0xFF0000);

        try {
            await this.dispatch('', {embed: embed});
        }
        catch (e) { 
            // if we don't have permissions here, we're kinda screwed, let's just
            // scream loud and hope our log gets read

            /**
             * Called when a non-recoverable permissions error occurs
             * 
             * @event CompilerClient#missingPermissions
             * @type {Guild} guild the permissions error occured in
             */
            this.message.client.emit('missingPermissions', this.message.guild)
        }
    }

    /**
     * Shortcut to this.message.channel.send()
     *
     * @param {StringResolvable} content
     * @param {MessageOptions | MessageEmbed | Attachment} [options={}]
     *
     * @return {Promise<Message | Message[]}
     */
    async dispatch(content, options) {
        try {
            return await this.message.channel.send(content, options)
        }
        catch (e) { 
            // if we don't have permissions here, we're kinda screwed, let's just
            // scream loud and hope our log gets read

            /**
             * Called when a non-recoverable permissions error occurs
             * 
             * @event CompilerClient#missingPermissions
             * @type {Guild} guild the permissions error occured in
             */
            this.message.client.emit('missingPermissions', this.message.guild, e.message)
        }
    }

    /**
     * Shortcut to this.message.author
     *
     * @return {Author}
     */
    getAuthor() {
        return this.message.author;
    }
    /**
     * Argument string (excluding command name)
     *
     * @return {string}
     */
    getArgString() {
        const rMatch = this.message.content.match(/(?:[^\s"]+|"[^"]*")+/g);

        if (rMatch == null) {
            return '';
        }

        return rMatch.slice(1).join(' ');
    }

    /**
     * Set a command object to the message
     *
     * @param {CompilerCommand} command - Command to set it to
     */
    setCommand(command) {
        this.command = command;
    }

    /**
     * Get array of arguments (minus the command name)
     *
     * @return {string[]}
     */
    getArgs() {
        let args = this.message.content.match(/(?:[^\s"]+|"[^"]*")+/g)

        args.shift()

        return args;
    }

    /**
     * Piece together remaining args as a string
     *
     * @param {number} index - Index of the arg to start joining together
     *
     * @return {string}
     */
    joinArgAfter(index) {
        return this.getArgs().slice(index).join(' ');
    }
}
