import { MessageEmbed } from 'discord.js'
import stripAnsi from 'strip-ansi';

import CompilerCommand from './utils/CompilerCommand';
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'
import SupportServer from './../SupportServer'
import CompileCommand from './compile'
import { GodboltSetup } from './../utils/Godbolt'
import DiscordMessageMenu from './../utils/DiscordMessageMenu'

export default class AsmCommand extends CompilerCommand {
    /**
     *  Creates the compile command
     * 
     * @param {CompilerClient} client
     */    
    constructor(client) {
        super(client, {
            name: 'asm',
            description: 'Outputs the assembly for the input code \nNote: This command\'s code input MUST be encapsulated in codeblocks',
            developerOnly: false
        });
    }

    /**
     * Function which is executed when the command is requested by a user
     *
     * @param {CompilerCommandMessage} msg
     */
    async run(msg) {
        const args = msg.getArgs();
        if (args.length < 1)
            return this.help(msg);

        if (args[0].toLowerCase() =='compilers') {
            args.shift();

            if (args.length < 1) {
                msg.replyFail(`You must input a valid language to view it's compilers \n\nUsage: ${this.client.prefix}asm compilers <language>`);
                return;
            }

            const lang = args[0]
            const language = this.client.godbolt.findLanguageByAlias(lang)
            if (language)
            {
                let lookupName = language.name.toLowerCase();
                let items = [];
                this.client.godbolt.compilers.forEach((compiler) => {
                    if (lookupName == compiler.lang.toLowerCase())
                        items.push(`${compiler.name}: **${compiler.id}**`);
                });

                let menu = new DiscordMessageMenu(msg.message, `Valid Godbolt '${lookupName}' compilers:`, 0x00FF00, 15, `Select a bold name on the right to use in place of the language in the ${this.client.prefix}asm command!`);
                menu.buildMenu(items);
                
                try {
                    await menu.displayPage(0);
                    return;
                }
                catch (error) {
                    msg.replyFail('Error with menu system, am I missing permissions?\n' + error);
                    return;
                }
            }
        }

		if (args.length < 1) {
			return await this.help(msg);
		}

        let lang = args[0].toLowerCase();
        args.shift();

        if (!this.client.godbolt.isValidCompiler(lang) && !this.client.godbolt.getDefaultCompiler(lang)) {
            msg.replyFail(`You must input a valid language or compiler \n\n Usage: ${this.client.prefix}asm <language/compiler> \`\`\`<code>\`\`\``);
            return;
        }

        const argsData = CompileCommand.parseArguments(args);
        let code = null;
        // URL request needed to retrieve code
        if (argsData.fileInput.length > 0) {
            try {
                code = await CompileCommand.getCodeFromURL(argsData.fileInput);
            }
            catch (e) {
                msg.replyFail(`Could not retrieve code from url \n ${e.message}`);
                return;
            }
        }
        // Standard ``` <code> ``` request
        else {
            code = CompileCommand.getCodeBlockFromText(msg.message.content);
            if (code) {
                code = CompileCommand.cleanLanguageSpecifier(code);
            }
            else {
                msg.replyFail('You must attach codeblocks containing code to your message');
                return;
            }
            const stdinblock = CompileCommand.getStdinBlockFromText(msg.message.content);
            if (stdinblock) {
                argsData.stdin = stdinblock;
            }
        }

        let reactionSuccess = false;
        if (this.client.loading_emote)
        {
            try {
                await msg.message.react(this.client.loading_emote);
                reactionSuccess = true;
            }
            catch (e) {
                msg.replyFail(`Failed to react to message, am I missing permissions?\n${e}`);
            }    
        }

        let setup = null;
        try {
            setup = new GodboltSetup(this.client.godbolt, code, lang, argsData.options);
        }
        catch (e) {
            msg.replyFail(`You must input a valid language or compiler \n\n Usage: ${this.toString()} <language/compiler> \`\`\`<code>\`\`\``);
            return;
        }

        let [errors, asm] = [null, null];  
        try {
            [errors, asm] = await setup.dispatch();
        }
        catch (e) {
            msg.replyFail(`Godbolt request failure \n${e.message} \nPlease try again later`);
            return;
        }

        //remove our react
        if (reactionSuccess && this.client.loading_emote) {
            try {
                await msg.message.reactions.resolve(this.client.loading_emote).users.remove(this.client.user);
            }
            catch (error) {
                msg.replyFail(`Unable to remove reactions, am I missing permissions?\n${error}`);
            }
        }   
        
        SupportServer.postAsm(code, lang, msg.message.author, msg.message.guild, errors==null, errors, this.client.compile_log, this.client.token);
        
        let embed = null;
        if (errors == null) {
            // Yeah we're just gonna hack godbolt onto our wandbox response builder
            embed = AsmCommand.buildResponseEmbed(msg, {
                status: 0,
                program_message: asm,
            });
        }
        else {
            embed = AsmCommand.buildResponseEmbed(msg, {
                status: 1,
                compiler_message: errors,
            });
        }

        let responsemsg = await msg.dispatch('', embed);
        try {
            responsemsg.react((embed.color == 0xFF0000)?'❌':'✅');
        }
        catch (error) {
            msg.replyFail(`Unable to react to message, am I missing permissions?\n${error}`);
            return;
        }
    }

    /**
     * Builds a compilation response embed
     * 
     * @param {CompilerCommandMessage} msg 
     * @param {*} json 
     */
    static buildResponseEmbed(msg, json) {
        const embed = new MessageEmbed()
        .setTitle('Assembly Results:')
        .setFooter("Requested by: " + msg.message.author.tag + " || Powered by godbolt.org")
        .setColor(0x00FF00);

        if (json.status) {
            if (json.status != 0) {
                embed.setColor((0xFF0000));
            }
            else {
                embed.setColor(0x00FF00);
                embed.addField('Status code', `Finished with exit code: ${json.status}`);    
            }
        }

        if (json.signal) {
            embed.addField('Signal', `\`\`\`${json.signal}\`\`\``);
        }

        if (json.url) {
            embed.addField('URL', `Link: ${json.url}`);
        }

        if (json.compiler_message) {
            if (json.compiler_message.length >= 1017) {
                json.compiler_message = json.compiler_message.substring(0, 1016);
            }
            /**
             * Certain compiler outputs use unicode control characters that
             * make the user experience look nice (colors, etc). This ruins
             * the look of the compiler messages in discord, so we strip them
             * out with stripAnsi()
             */
            json.compiler_message = stripAnsi(json.compiler_message);
            embed.addField('Compiler Output', `\`\`\`${json.compiler_message}\n\`\`\`\n`);
        }

        if (json.program_message) {
            /**
             * Annoyingly, people can print '`' chars and ruin the formatting of our
             * program output. To counteract this, we can place a unicode zero-width
             * character to escape it.
             */
            json.program_message = json.program_message.replace(/`/g, "\u200B"+'`');

            // This kinda sucks, to show full assembly output we'll need to split our fields into
            // reasonbly-sized chunks. Sanity resumes after this if statement.
            let message = json.program_message;
            let parts = []
            if (message.length > 1012) {
                while (message.length > 1012) {
                    let nearest_newline = 0;
                    for(let i = 1012; i > 0; i--) {
                        if (message[i] == '\n') {
                            nearest_newline = i;
                            break;
                        }
                    }
    
                    let substr = message.substring(0, nearest_newline+1);
                    parts.push(substr);
                    message = message.substring(nearest_newline);
                }
                parts.push(message);

                let count = 1;
                for (const part of parts) {
                    part = stripAnsi(part);
                    embed.addField(`Assembly Output Pt. ${count++}`, `\`\`\`x86asm\n${part}\`\`\``);
                }
                return embed;
            }
            json.program_message = stripAnsi(json.program_message);

            embed.addField('Assembly Output', `\`\`\`x86asm\n${json.program_message}\`\`\``);
        }
        return embed;
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
            .addField('Standard compile', `${this.toString()} <language|compiler> \\\`\\\`\\\`<code>\\\`\\\`\\\``)
            .addField('Compile w/ options', `${this.toString()} <language|compiler> <options> \\\`\\\`\\\`<code>\\\`\\\`\\\``)
            .addField('Compile w/ stdin', `${this.toString()} <language|compiler> | <stdin> \\\`\\\`\\\`<code>\\\`\\\`\\\``)
            .addField('Compile w/ url code', `${this.toString()} <language|compiler> < http://online.file/url`)
            .addField('Search godbolt compilers', `${this.toString()} compilers <language>`)
            .setThumbnail('https://imgur.com/TNzxfMB.png')
            .setFooter(`Requested by: ${message.message.author.tag}`)
        return await message.dispatch('', embed);
    }
}
