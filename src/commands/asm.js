import { MessageEmbed } from 'discord.js'
import stripAnsi from 'strip-ansi';

import CompilerCommand from './utils/CompilerCommand';
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'
import SupportServer from './../SupportServer'
import { Godbolt, GodboltSetup } from './../utils/apis/Godbolt'
import DiscordMessageMenu from './../utils/DiscordMessageMenu'
import CompilationParser from './utils/CompilationParser';

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

            await AsmCommand.handleCompilers(args, msg, this.client.godbolt);
            return;
        }
        if (args[0].toLowerCase().includes('language')) {
            await AsmCommand.handleLanguage(msg, this.client.godbolt);
            return;
        }

        let lang = args[0].toLowerCase();
        args.shift();

        let godbolt = this.client.godbolt;
        if (!godbolt.isValidCompiler(lang) && !godbolt.isValidLanguage(lang)) {
            msg.replyFail(`"${lang}" is not a supported Godbolt language or compiler!`);
            return;
        }
        
        let parser = new CompilationParser(msg);

        const argsData = parser.parseArguments();
        let code = null;
        // URL request needed to retrieve code
        if (argsData.fileInput.length > 0) {
            try {
                code = await CompilationParser.getCodeFromURL(argsData.fileInput);
            }
            catch (e) {
                msg.replyFail(`Could not retrieve code from url \n ${e.message}`);
                return;
            }
        }
        // Standard ``` <code> ``` request
        else {
            code = parser.getCodeBlockFromText();
            if (code) {
                code = CompilationParser.cleanLanguageSpecifier(code);
            }
            else {
                msg.replyFail('You must attach codeblocks containing code to your message');
                return;
            }
            const stdinblock = parser.getStdinBlockFromText();
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
                
        let embed = null;
        if (errors == null) {
            // Yeah we're just gonna hack godbolt onto our wandbox-style response builder
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

        SupportServer.postAsm(code, lang, msg.message.author, msg.message.guild, errors==null, errors, this.client.compile_log, this.client.token);

        let responsemsg = await msg.dispatch('', embed);
        if (!responsemsg) {
            if (this.client.github_link) {
                msg.replyFail(`Message dispatch failure, I am either missing permissions or the response message was unable to be sent.\n\nIf you believe this is an error, please create a bug report [here](${this.client.github_link}/issues)`);
                return;    
            }
            else {
                msg.replyFail(`Message dispatch failure, I am either missing permissions or the response message was unable to be sent.`);
                return;    
            }
        }
        try {
            responsemsg.react((embed.color == 0xFF0000)?'❌':'✅');
        }
        catch (error) {
            msg.replyFail(`Unable to react to message, am I missing permissions?\n${error}`);
            return;
        }
    }

    /**
     * Handles the languages list sub-command
     * @param {CompilerCommandMessage} msg
     * @param {Godbolt} godbolt
     */
    static async handleLanguage(msg, godbolt) {
        let items = [];
        godbolt.forEach((language) => items.push(`${language.id}`));

        let menu = new DiscordMessageMenu(msg.message, `Valid Godbolt languages:`, 0x00FF00, 15);
        menu.buildMenu(items);
        
        try {
            await menu.displayPage(0);
        }
        catch (error) {
            msg.replyFail('Error with menu system, am I missing permissions?\n' + error);
        }
    }

    /**
     * Handles the compilers list sub-command
     * @param {string[]} args
     * @param {CompilerCommandMessage} msg
     * @param {Godbolt} godbolt
     */
    static async handleCompilers(args, msg, godbolt) {
        let prefix = msg.message.client.prefix;
        if (args.length < 1) {
            msg.replyFail(`You must input a valid language to view it's compilers \n\nUsage: ${prefix}asm compilers <language>`);
            return;
        }

        const language = godbolt.findLanguageByAlias(args[0]);
        if (!language)
        {
            msg.replyFail(`"${args[0]}" is not a valid language,  use the \`${prefix}asm languages\` command to select a valid one!`);
            return;
        }

        let items = [];
        language.forEach((compiler) => items.push(`${compiler.name}: **${compiler.id}**`));

        let menu = new DiscordMessageMenu(msg.message, `Valid Godbolt '${language.name}' compilers:`, 0x00FF00, 15, `Select a bold name on the right to use in place of the language in the ${prefix}asm command!`);
        menu.buildMenu(items);
        
        try {
            await menu.displayPage(0);
        }
        catch (error) {
            msg.replyFail('Error with menu system, am I missing permissions?\n' + error);
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
            if (message.length > 1010) {
                let count = 1;
                while (message.length > 1010 && embed.length+1024< 6000) {
                    let nearest_newline = 0;
                    for(let i = 1010; i > 0; i--) {
                        if (message[i] == '\n') {
                            nearest_newline = i;
                            break;
                        }
                    }

                    let substr = message.substring(0, nearest_newline+1);
                    substr = stripAnsi(substr);
                    embed.addField(`Assembly Output Pt. ${count++}`, `\`\`\`x86asm\n${substr}\`\`\``);
                    message = message.substring(nearest_newline);
                }
                if (embed.length+message.length+13 < 6000) {
                    message = stripAnsi(message);
                    embed.addField(`Assembly Output Pt. ${count++}`, `\`\`\`x86asm\n${message}\`\`\``);    
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
            .addField('Search godbolt languages', `${this.toString()} languages`)
            .setThumbnail('https://imgur.com/TNzxfMB.png')
            .setFooter(`Requested by: ${message.message.author.tag}`)
        return await message.dispatch('', embed);
    }
}
