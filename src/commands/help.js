import { Message, MessageEmbed, Client } from 'discord.js'
import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'

export default class HelpCommand extends CompilerCommand {
    /**
     *  Creates the Compile command
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
                return await msg.replyFail(`Command: ${command} not found!`);
            }

            let cmd = this.client.commands.get(command);
            if (cmd.developerOnly) {
                return await msg.replyFail(`Command: ${command} not found!`);
            }

            await cmd.help(msg);
        }
        // Nothing given to us, show the full list
        else {
            const embed = new MessageEmbed()
                .setDescription(`**For more information on how to use a command, try \ntyping ${this.client.prefix}help <command name>**\n\n Struggling? Check out our wiki: https://github.com/Headline/discord-compiler-bot/wiki`)
                .setTitle('Command list')
                .setFooter(`Requested by: ${msg.message.author.tag}`)
                .setColor(0x00FF00)
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
            .setColor(0x00FF00)
            .addField('Command-based help', `${this.toString()} <command name>`)
            .setThumbnail('https://imgur.com/TNzxfMB.png')
            .setFooter(`Requested by: ${message.message.author.tag}`)
        return await message.dispatch('', embed);
    }
}
