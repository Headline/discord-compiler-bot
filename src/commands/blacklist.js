import { Message, MessageEmbed, Client } from 'discord.js'
import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import DiscordMessageMenu from '../utils/DiscordMessageMenu'

export default class BlacklistCommand extends CompilerCommand {
    /**
     *  Creates the Compile command
     */
    constructor(client) {
        super(client, {
            name: 'blacklist',
            description: 'Blacklists a guild from sending requiests',
            developerOnly: true,
        });
    }

    /**
     * Function which is executed when the command is requested by a user
     *
     * @param {CompilerCommandMessage} msg
     */
    async run(msg) {
        const args = msg.getArgs();
        const guild = args[0];

        if (this.client.messagerouter.blacklist.isBlacklisted(guild))
            return await msg.replyFail('Specified guild is already blacklisted');

        await this.client.messagerouter.blacklist.blacklist(guild);

        const embed = new MessageEmbed()
            .setTitle('Guild Blacklisted')
            .setDescription(`${guild} has been blacklisted`)
            .setThumbnail('https://imgur.com/KXZqNWq.png')
            .setColor(0xFF0000)
            .setFooter(`Requested by: ${msg.message.author.tag}`)
        await msg.dispatch('', embed);

    }

    /**
     * Displays the help information for the given command
     *
     * @param {CompilerCommandMessage} message
     */
    async help(message) {
    }
}