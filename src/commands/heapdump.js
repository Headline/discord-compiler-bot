import { MessageEmbed } from 'discord.js'

import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'

import heapdump from 'heapdump';

export default class HeapdumpCommand extends CompilerCommand {
    /**
     *  Creates the blacklist command
     * 
     * @param {CompilerClient} client
     */    
    constructor(client) {
        super(client, {
            name: 'heapdump',
            description: 'Dumps the bot\'s heap for inspection',
            developerOnly: true,
        });
    }

    /**
     * Function which is executed when the command is requested by a user
     *
     * @param {CompilerCommandMessage} msg
     */
    async run(msg) {
        heapdump.writeSnapshot((err, string) => {
            if (err) {
                msg.dispatch(err.message);
                return;
            }
            
            const embed = new MessageEmbed()
            .setTitle('Chrome Heapdump')
            .setDescription(`Written to file:\n \`${string}\``)
            .setThumbnail('https://imgur.com/PVBdOYi.png')
            .setColor(0x046604)
            .setFooter(`Requested by: ${msg.message.author.tag}`)
            msg.dispatch('', embed);
        });
    }
}