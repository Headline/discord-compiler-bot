import { MessageEmbed } from 'discord.js'

import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'
import DiscordMessageMenu from '../utils/DiscordMessageMenu'

export default class LanguagesCommand extends CompilerCommand {
    /**
     *  Creates the language command
     * 
     * @param {CompilerClient} client
     */
    constructor(client) {
        super(client, {
            name: 'languages',
            description: 'Displays all supported languages',
            developerOnly: false
        });
    }

    /**
     * Function which is executed when the command is requested by a user
     *
     * @param {CompilerCommandMessage} msg
     */
    async run(msg) {
        let langs = this.client.wandbox.keyArray();
        let menu = new DiscordMessageMenu(msg.message, `Supported languages:`, 0x00FF00, 15);
        menu.buildMenu(langs);

        try {
            await menu.displayPage(0);
        }
        catch (error) {
            msg.replyFail('Error with menu system, am I missing permissions?\n' + error);
        }
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
            .addField('Show all languages', `${this.toString()}`)
            .setThumbnail('https://imgur.com/TNzxfMB.png')
            .setFooter(`Requested by: ${message.message.author.tag}`)
        return await message.dispatch('', embed);
    }
}