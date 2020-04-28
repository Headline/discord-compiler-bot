import { MessageEmbed } from 'discord.js'

import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'

export default class InviteCommand extends CompilerCommand {
    /**
     *  Creates the invite command
     * 
     * @param {CompilerClient} client
     */    
    constructor(client) {
        super(client, {
            name: 'invite',
            description: 'Grabs the bot\'s invite link',
            developerOnly: false,
        });
    }

    /**
     * Function which is executed when the command is requested by a user
     *
     * @param {CompilerCommandMessage} msg
     */
    async run(msg) {
        const embed = new MessageEmbed()
            .setTitle('Invite Link')
            .setDescription(`Click the link below to invite me to your server!\n\n` 
            + `[Invite me!](${this.client.invite_link})`)
            .setThumbnail('https://i.imgur.com/CZFt69d.png')
            .setColor(0x00FF00)
            .setFooter(`Requested by: ${msg.message.author.tag}`)
        await msg.dispatch('', embed);
    }

   /**
     * Displays the help information for the given command
     *
     * @param {CompilerCommandMessage} message
     */
    async help(message) {
        const embed = new MessageEmbed()
            .setTitle('Command Usage')
            .setDescription(`*${this.description}*`)
            .setColor(0x00FF00)
            .addField('Get invite link', `${this.toString()}`)
            .setThumbnail('https://imgur.com/TNzxfMB.png')
            .setFooter(`Requested by: ${message.message.author.tag}`)
        return await message.dispatch('', embed);
    }
}