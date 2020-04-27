import url from 'url';
import { MessageEmbed } from 'discord.js'
import stripAnsi from 'strip-ansi';
import fetch from 'node-fetch';

import CompilerCommand from './utils/CompilerCommand';
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'
import { Compiler, CompileSetup } from '../utils/Wandbox';

export default class CompileCommand extends CompilerCommand {
    /**
     *  Creates the compile command
     * 
     * @param {CompilerClient} client
     */    
    constructor(client) {
        super(client, {
            name: 'compile',
            description: 'Compiles a script \nNote: This command\'s code input MUST be encapsulated in codeblocks',
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
		
		if (args.length < 1) {
			return await this.help(msg);
		}
		
        let lang = args[0].toLowerCase();
        args.shift();

        if (!this.client.compilers.isValidCompiler(lang) && !this.client.compilers.has(lang)) {
            msg.replyFail(`You must input a valid language or compiler \n\n Usage: ${this.client.prefix}compile <language/compiler> \`\`\`<code>\`\`\``);
            return;
        }

        const argsData = this.parseArguments(args);
        let code = null;
        // URL request needed to retrieve code
        if (argsData.fileInput.length > 0) {
            try {
                code = await this.getCodeFromURL(argsData.fileInput);
            }
            catch (e) {
                msg.replyFail(`Could not retrieve code from url \n ${e.message}`);
                return;
            }
        }
        // Standard ``` <code> ``` request
        else {
            code = this.getCodeBlockFromText(msg.message.content);
            if (code) {
                code = this.cleanLanguageSpecifier(code);
            }
            else {
                msg.replyFail('You must attach codeblocks containing code to your message');
                return;
            }
            const stdinblock = this.getStdinBlockFromText(msg.message.content);
            if (stdinblock) {
                argsData.stdin = stdinblock;
            }
        }

        let setup = new CompileSetup(code, lang, argsData.stdin, true, argsData.options, this.client.compilers);
        let comp = new Compiler(setup);

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

        let json = null;
        try {
            json = await comp.compile();
        }
        catch (e) {
            msg.replyFail(`Wandbox request failure \n ${e.message} \nPlease try again later`);
            return;
        }
        if (!json) {
            msg.replyFail(`Invalid Wandbox response \nPlease try again later`);
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

		this.client.supportServer.postCompilation(code, lang, json.url, msg.message.author, msg.message.guild, json.status == 0, json.compiler_message);
        this.client.stats.compilationExecuted(lang);

        let embed = this.buildResponseEmbed(msg, json);
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
    buildResponseEmbed(msg, json) {
        const embed = new MessageEmbed()
        .setTitle('Compilation Results:')
        .setFooter("Requested by: " + msg.message.author.tag + " || Powered by wandbox.org")
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
            if (json.program_message.length >= 1017) {
                json.program_message = json.program_message.substring(0, 1016);
            }

            json.program_message = stripAnsi(json.program_message);

            /**
             * Annoyingly, people can print '`' chars and ruin the formatting of our
             * program output. To counteract this, we can place a unicode zero-width
             * character to escape it.
             */
            json.program_message = json.program_message.replace(/`/g, "\u200B"+'`');
            embed.addField('Program Output', `\`\`\`\n${json.program_message}\`\`\``);
        }
        return embed;
    }

    /**
     * Parses the code from the input text
     * 
     * @param {string} text 
     * @return {string} null if no block found
     */
    getCodeBlockFromText(text) {
        const regex = /```([\s\S]*?)```/g;
        let match = regex.exec(text);
        if (!match)
            return null;

        // If we match again, then our code belongs in the new match
        let block1 = match[1].trim();
        match = regex.exec(text);
        if (!match)
            return block1;
        else
            return match[1].trim();
    }

    /**
     * Parses the stdin block from the input text
     * 
     * @param {string} text 
     * @return {string} null if no block found
     */
    getStdinBlockFromText(text) {
        const regex = /```([\s\S]*?)```/g;
        let match = regex.exec(text);
        if (!match)
            return null;

        // If we match again, our stdin belongs in the first match result
        let block1 = match[1].trim();
        match = regex.exec(text);
        if (!match)
            return null;
        else
            return block1;
    }

    async getCodeFromURL(rawUrl) {
        try {
            let fileInput = url.parse(rawUrl);

            if (!fileInput.hostname || !fileInput.protocol) {
                // TODO: error malformed url
            }
    
            let response = await fetch(fileInput.href);
            let data = await response.text();
            if (!response.ok) {
                throw new Error(`Error requesting online code URL - ${response.status}\n ${data}`);
            }

            return data;
        }
        catch (error) {
            throw (error); // rethrow to higher level
        }
    }

    /**
     * Removes the language specifier from the input string
     * 
     * @param {string} code 
     * @return {string} cleaned code string
     */
    cleanLanguageSpecifier(code) {
        const discordLanguages = [ "1c", "abnf", "accesslog", "actionscript", "ada", "apache", "applescript",
        "arduino", "armasm", "asciidoc", "aspectj", "autohotkey", "autoit", "avrasm",
        "awk", "axapta", "bash", "basic", "bnf", "brainfuck", "bf", "c", "cal", "capnproto", "ceylon",
        "clean", "clojure-repl", "clojure", "cmake", "coffeescript", "coq", "cos",
        "cpp", "crmsh", "crystal", "cs", "csharp", "csp", "css", "d", "dart", "delphi", "diff",
        "django", "dns", "dockerfile", "dos", "dsconfig", "dts", "dust", "ebnf",
        "elixir", "elm", "erb", "erlang-repl", "erlang", "excel", "fix", "flix", "fortran",
        "fsharp", "gams", "gauss", "gcode", "gherkin", "glsl", "go", "golo", "gradle", "groovy",
        "haml", "handlebars", "haskell", "haxe", "hsp", "htmlbars", "http", "hy", "inform7",
        "ini", "irpf90", "java", "javascript", "jboss-cli", "json", "js", "julia-repl", "julia",
        "kotlin", "lasso", "ldif", "leaf", "less", "lisp", "livecodeserver", "livescript",
        "llvm", "lsl", "lua", "makefile", "markdown", "mathematica", "matlab", "maxima",
        "mel", "mercury", "mipsasm", "mizar", "mojolicious", "monkey", "moonscript", "n1ql",
        "nginx", "nimrod", "nix", "nsis", "objectivec", "ocaml", "openscad", "oxygene",
        "parser3", "perl", "pf", "php", "pony", "powershell", "processing", "profile",
        "prolog", "protobuf", "puppet", "purebasic", "python", "py", "q", "qml", "r", "rib",
        "roboconf", "routeros", "rsl", "ruby", "ruleslanguage", "rust", "scala", "scheme",
        "scilab", "scss", "shell", "smali", "smalltalk", "sml", "sqf", "sql", "stan", "stata",
        "step21", "stylus", "subunit", "swift", "taggerscript", "tap", "tcl", "tex", "thrift",
        "tp", "twig", "typescript", "vala", "vbnet", "vbscript-html", "vbscript", "verilog",
        "vhdl", "vim", "x86asm", "xl", "xml", "xquery", "yaml", "zephir"
        ];
    
        let stop = 0;
        while (code.charAt(stop) != '\n' && code.charAt(stop) != ' ' && stop < code.length) {
            stop++;
        }
    
        let substr = code.substr(0, stop);
        for (let i = 0; i < discordLanguages.length; i++) {
            if (substr.toLowerCase() == discordLanguages[i]) {
                code = code.replace(substr, '');
                break;
            }
        }
        return code;
    }

    /**
     * Figure out the args passed, it should go in the following order:
     * compile <language> < http://online.file/url | pipe data here
     * Both online file input and pipe data are optional. If the redirect input
     * character happens after pipe character just assume it's not file input
     * 
     * @param {string[]} args 
     */
    parseArguments(args) {
        let argsData = {
            options: "",
            fileInput: "",
            stdin: "",
        };

        while (args.length > 0) {
            let current = args[0];

            // we encountered codeblocks, no further parsing necessary
            if (current.includes('```')) {
                break;
            }
            // accept code input via redirection operator
            else if (current == '<') {
                argsData.fileInput = args[1];
                args.shift();
            }
            // pipe operator should be last, everything after it should be stdin
            else if (current == '|') {
                do {
                    args.shift(); // kill previous
                    current = args[0];
                    argsData.stdin += current + ' ';
                } while (args.length > 1 && !args[1].includes('```'));
                args = []; // stop parsing
            }
            // anything else should just be an option
            else {
                argsData.options += current + ' ';
            }

            args.shift();
        }

        // cleanup whitespace
        argsData.options = argsData.options.trim();
        argsData.fileInput = argsData.fileInput.trim();
        argsData.stdin = argsData.stdin.trim();

        return argsData;
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
            .setThumbnail('https://imgur.com/TNzxfMB.png')
            .setFooter(`Requested by: ${message.message.author.tag}`)
        return await message.dispatch('', embed);
    }

}
