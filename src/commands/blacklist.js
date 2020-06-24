import { MessageEmbed } from 'discord.js'

import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'

export default class BlacklistCommand extends CompilerCommand {
    /**
     *  Creates the blacklist command
     * 
     * @param {CompilerClient} client
     */    
    constructor(client) {
        super(client, {
            name: 'blacklist',
            description: 'Blacklists a guild or a user from sending requests',
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
            return await msg.replyFail('You must supply a guild or user to blacklist!');

        const guild = args[0];

        if (isNaN(guild))
            return await msg.replyFail('Specified snowflake is invalid');

        if (this.client.messagerouter.blacklist.isBlacklisted(guild))
            return await msg.replyFail('Specified snowflake is already blacklisted');

        await this.client.messagerouter.blacklist.blacklist(guild);

        // lets update all blacklists with our newest blacklist
        this.client.shard.broadcastEval(`this.messagerouter.blacklist.addToCache('${guild}')`);

        const embed = new MessageEmbed()
            .setTitle('Snowflake Blacklisted')
            .setDescription(`${guild} has been blacklisted`)
            .setThumbnail('https://imgur.com/PVBdOYi.png')
            .setColor(0xFF0000)
            .setFooter(`Requested by: ${msg.message.author.tag}`)
        await msg.dispatch('', embed);

    }
}