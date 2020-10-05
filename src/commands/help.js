import { MessageEmbed } from 'discord.js'

import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'

export default class HelpCommand extends CompilerCommand {
    /**
     *  Creates the help command
     * 
     * @param {CompilerClient} client
     */
    constructor(client) {
        super(client, {
            name: 'help',
            description: 'Displays information about how to use the compiler',
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

        // Lookup command by name if we got a name
        if (args.length > 0) {
            const command = args[0].toLowerCase();

            if (!this.client.commands.has(command)) {
                msg.replyFail(`Command: ${command} not found!`);
                return;
            }

            let cmd = this.client.commands.get(command);
            if (cmd.developerOnly) {
                msg.replyFail(`Command: ${command} not found!`);
                return;
            }

            await cmd.help(msg);
        }
        // Nothing given to us, show the full list
        else {
            const embed = new MessageEmbed()
                .setDescription(`**For more information on how to use a command, try \ntyping ${this.client.prefix}help <command name>**\n\n Struggling? Check out our wiki: https://github.com/Headline/discord-compiler-bot/wiki`)
                .setTitle('Command list')
                .setFooter(`Requested by: ${msg.message.author.tag}`)
                .setColor(0x046604)
                .setThumbnail('https://imgur.com/TNzxfMB.png')
                .setFooter(`Requested by: ${msg.message.author.tag}`)


            for (const command of this.client.commands.array()) {
                if (command.developerOnly)
                    continue;

                if (command.name != 'help')
                    embed.addField(command.toString(), `\`\`\`${command.description}\`\`\``);
            }
            await msg.dispatch('', embed);
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
            .setColor(0x046604)
            .addField('Command-based help', `${this.toString()} <command name>`)
            .setThumbnail('https://imgur.com/TNzxfMB.png')
            .setFooter(`Requested by: ${message.message.author.tag}`)
        return await message.dispatch('', embed);
    }
}
