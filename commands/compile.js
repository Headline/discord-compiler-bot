const url = require("url");
const fetch = require("node-fetch");
const Discord = require('discord.js');
const WandBox = require ('../WandBox.js');
const stripAnsi = require('strip-ansi');
const botconfig = require('../settings.json');

function cleanControlChars(dirty) {
    return stripAnsi(dirty);
}

module.exports.run = async (client, message, args, prefix, compilerAPI, SupportServer) => {

    let lang = args[1].toLowerCase();
    if (compilerAPI.languages.indexOf(lang) < 0
    &&  !compilerAPI.isValidCompiler(lang))
    {
        const embed = new Discord.RichEmbed()
        .setTitle('Error:')
        .setColor(0xFF0000)
        .setDescription(`You must supply a valid language or compiler as an argument!\n`
                        + `Usage: ${prefix}compile <lang/compiler> \`\`\` <code> \`\`\``)
        message.channel.send(embed).catch();
        return;
    }

    // Figure out the args passed, it should go in the following order:
    // compile <language> < http://online.file/url | pipe data here
    // Both online file input and pipe data are optional. If the redirect input
    // character happens after pipe character just assume it's not file input
    let argsData = {
        options: "",
        fileInput: "",
        fileInputReached: false,
        stdin: "",
        stdinReached: false,
    };
    for (let i = 2; i < args.length; i++) {
        if (args[i].indexOf('```') > -1) break;

        if (args[i] === "|") {
            argsData.stdinReached = true;
            continue;
        } else if (args[i] === "<") {
            argsData.fileInputReached = true;
            continue;
        }

        if (argsData.stdinReached) {
            argsData.stdin += args[i] + " ";
        } else if (argsData.fileInputReached) {
            argsData.fileInput += args[i] + " ";
        } else {
            argsData.options += args[i] + " ";
        }
    }
    options = argsData.options.trim();
    let fileInput = url.parse(argsData.fileInput.trim());
    let stdin = argsData.stdin.trim();

    let code = null;

    if (fileInput.protocol && fileInput.hostname) {
        try {
            let response = await fetch(fileInput.href);
            let data = await response.text();
            if (!response.ok) {
                throw new Error(
                    `Error requesting online code URL - ${response.status}\n ${data}`
                );
            }
            code = data;
        } catch (e) {
            console.error("There was some error with online code input", e.message);
        }
    };

    let regex = /```([\s\S]*?)```/g;
    let match = regex.exec(message.content);

    if (!code) {
        // If we reach this it means no online file input
        if (!match)
        {
            const embed = new Discord.RichEmbed()
            .setTitle('Error:')
            .setColor(0xFF0000)
            .setDescription(`You must attach codeblocks containing code to your message`)
            message.channel.send(embed).catch();
            return;
        }

        code = match[1].trim();
        if (code.length <= 0)
        {
            const embed = new Discord.RichEmbed()
            .setTitle('Error:')
            .setColor(0xFF0000)
            .setDescription(`You must actually supply code to compile!`)
            message.channel.send(embed).catch();
            return;
        }
    }

    let match2 = regex.exec(message.content);
    if (match2) { // two codeblocks? wtf?
        code = match2[1].trim();
        stdin = match[1].trim();
    }

    let discordLanguages = [ "1c", "abnf", "accesslog", "actionscript", "ada", "apache", "applescript",
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

    let setup = new WandBox.CompileSetup(code, lang, stdin, true, options, compilerAPI);
    let comp = new WandBox.Compiler(setup);
    let loading = client.emojis.get(botconfig.loading_emote);
    message.react(loading).then((msg) => {
        comp.compile((json) => {
            message.clearReactions().catch();

            const embed = new Discord.RichEmbed()
            .setTitle('Compilation Results:')
            .setFooter("Requested by: " + message.author.tag
            + " || Powered by wandbox.org")
            .setColor(0x00FF00);

            /* The request failed */
            if (json == null) {
                embed.setColor(0xFF0000);
                embed.setDescription("It appears that a request has failed. It has either timed out or wandbox.org is rejecting requests. Please try again later.");
                message.channel.send(embed).catch();
                return;
            }

            /**
             * We got something back, build embed. Our strategy here is to
             * just insert whatever we get back. Whatever the API gives us,
             * we'll show.
             * TODO: will we ever get nothing? does that mean the world is ending?
             */
            if (json.hasOwnProperty('status')) {
                if (json.status != 0)
                    embed.setColor(0xFF0000);
                embed.addField('Status code', `Finished with exit code: ${json.status}`);
            }
            if (json.hasOwnProperty('signal'))
                embed.addField('Signal', `\`\`\`${json.signal}\`\`\``);
            if (json.hasOwnProperty('url'))
                embed.addField('URL', `Link: ${json.url}`);
            if (json.hasOwnProperty('compiler_message')) {
                if (json.compiler_message.length >= 1017) {
                    json.compiler_message = json.compiler_message.substring(0, 1016);
                }
                /**
                 * Certain compiler outputs use unicode control characters that
                 * make the user experience look nice (colors, etc). This ruins
                 * the look of the compiler messages in discord, so we strip them
                 * out with cleanControlChars()
                 */
                json.compiler_message = cleanControlChars(json.compiler_message);
                embed.addField('Compiler Output', `\`\`\`${json.compiler_message}\n\`\`\`\n`);
            }
            if (json.hasOwnProperty('program_message')) {
                if (json.program_message.length >= 1017) {
                    json.program_message = json.program_message.substring(0, 1016);
                }

                json.program_message = cleanControlChars(json.program_message);

                /**
                 * Annoyingly, people can print '`' chars and ruin the formatting of our
                 * program output. To counteract this, we can place a unicode zero-width
                 * character to escape it.
                 */
                json.program_message = json.program_message.replace(/`/g, "\u200B"+'`');
                embed.addField('Program Output', `\`\`\`\n${json.program_message}\`\`\``);
            }
            SupportServer.postCompilation(code, lang, json.url, message.author, message.guild, json.status == 0, json.compiler_message);
            message.channel.send(embed).then((msg) => {


                if (json.status == 0)
                    msg.react('✅').catch();
                else
                    msg.react('❌').catch();
            }).catch();
        });
    }).catch();
}

module.exports.help = {
    name:"compile",
    description:"compiles given code",
    dev: false
}
