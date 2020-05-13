import { MessageEmbed } from 'discord.js'

import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'
import DiscordMessageMenu from '../utils/DiscordMessageMenu'
import AsmCommand from './asm'

export default class CompilersCommand extends CompilerCommand {
    /**
     *  Creates the compilers command
     * 
     * @param {CompilerClient} client
     */    
    constructor(client) {
        super(client, {
            name: 'compilers',
            description: 'Displays the compilers for the specified language',
            developerOnly: false
        });
    }

    /**
     * Function which is executed when the command is requested by a user
     *
     * @param {CompilerCommandMessage} msg
     */
    async run(msg) {
        let args = msg.getArgs();

        if (args.length == 0) {
            return this.help(msg);
        }
        if (args[0].toLowerCase() =='asm') {
            args.shift();

            await AsmCommand.handleCompilers(args, msg, this.client.godbolt);
            return;
        }

        let langs = this.client.wandbox.getCompilers(args[0].toLowerCase()); 
        if (!langs) {
            msg.replyFail(`The language *\'${args[0]}\'* is either not supported, or you have accidentially typed in the wrong language.` 
            + `Try using the *${this.client.prefix}languages* command to see supported languages!`);
            return;
        }
        let menu = new DiscordMessageMenu(msg.message, `Supported \'${args[0].toLowerCase()}\' compilers:`, 0x00FF00, 15);
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
            .addField('Show compiler list', `${this.toString()} <language>`)
            .setThumbnail('https://imgur.com/TNzxfMB.png')
            .setFooter(`Requested by: ${message.message.author.tag}`)
        return await message.dispatch('', embed);
    }
}