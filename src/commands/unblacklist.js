import { MessageEmbed } from 'discord.js'

import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'

export default class UnBlacklistCommand extends CompilerCommand {
    /**
     *  Creates the unblacklist command
     * 
     * @param {CompilerClient} client
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

        if (args.length != 1)
            return await msg.replyFail('You must supply a guild to unblacklist!');

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
}