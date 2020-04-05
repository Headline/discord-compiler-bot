import { Message, MessageEmbed, Client } from 'discord.js'
import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import DiscordMessageMenu from '../utils/DiscordMessageMenu'

export default class UnBlacklistCommand extends CompilerCommand {
    /**
     *  Creates the Compile command
     */
    constructor(client) {
        super(client, {
            name: 'unblacklist',
            description: 'Unblacklists a guild',
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

        if (!this.client.messagerouter.blacklist.isBlacklisted(guild))
            return await msg.replyFail('Specified guild is not blacklisted');

        await this.client.messagerouter.blacklist.unblacklist(guild);

        const embed = new MessageEmbed()
            .setTitle('Guild Unblacklisted')
            .setDescription(`${guild} has been unblacklisted`)
            .setThumbnail('https://imgur.com/PVBdOYi.png')
            .setColor(0x99CCFF)
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