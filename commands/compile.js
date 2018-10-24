const Discord = require('discord.js');
const WandBox = require ('../WandBox.js');
const stripAnsi = require('strip-ansi');

function cleanControlChars(dirty) {
    return stripAnsi(dirty);
}

module.exports.run = async (client, message, args, prefix, compilerAPI) => {

    let match = message.content.match('```([\\s\\S]*)```');
    if (!match)
    {
        const embed = new Discord.RichEmbed()
        .setTitle('Error:')
        .setColor(0xFF0000)
        .setDescription(`You must attach codeblocks containing code to your message`)
        message.channel.send(embed);
        return;
    }

    let code = message.content.match('```([\\s\\S]*)```')[1].trim();
    if (code.length <= 0)
    {
        const embed = new Discord.RichEmbed()
        .setTitle('Error:')
        .setColor(0xFF0000)
        .setDescription(`You must actually supply code to compile!`)
        message.channel.send(embed);
        return;
    }

    let lang = args[1].toLowerCase();
    if (compilerAPI.languages.indexOf(lang) < 0
    &&  !compilerAPI.isValidCompiler(lang))
    {
        const embed = new Discord.RichEmbed()
        .setTitle('Error:')
        .setColor(0xFF0000)
        .setDescription(`You must supply a valid language or compiler as an argument!\n`
                        + `Usage: ${prefix}compile <lang/compiler> \`\`\` <code> \`\`\``)
        message.channel.send(embed);
        return;
    }

    let discordLanguages = [ 'asciidoc', 'autohotkey', 'bash', 'coffeescript', 'cpp',
    'cs', 'css', 'diff', 'fix', 'glsl', 'ini', 'json', 'md', 'ml', 'prolog', 'py',
    'tex', 'xml', 'xl'];
    for (let i = 0; i < discordLanguages.length; i++) {
        if (code.startsWith(discordLanguages[i])) {
            code = code.replace(discordLanguages[i], '');
            break;
        }
    }

    let setup = new WandBox.CompileSetup(code, lang, "", true, compilerAPI);
    let comp = new WandBox.Compiler(setup);
    let loading = client.emojis.get('504515210455941120');
    message.react(loading).then((msg) => {
        comp.compile((json) => {
            message.clearReactions();
            
            const embed = new Discord.RichEmbed()
            .setTitle('Compilation Results:')
            .setFooter("Requested by: " + message.author.tag
            + " || Powered by wandbox.org")
            .setColor(0x00FF00);

            if (json.hasOwnProperty('status')) {
                if (json.status != 0)
                    embed.setColor(0xFF0000);
                embed.addField('Status code', `Finished with exit code: ${json.status}`);
            }
            if (json.hasOwnProperty('url'))
                embed.addField('URL', `Link: ${json.url}`);
            if (json.hasOwnProperty('compiler_message')) {
                if (json.compiler_message.length >= 1017) {
                    json.compiler_message = json.compiler_message.substring(0, 1016);
                }
                json.compiler_message = cleanControlChars(json.compiler_message);
                embed.addField('Compiler Output', `\`\`\`${json.compiler_message}\n\`\`\`\n`);
            }
            if (json.hasOwnProperty('program_message')) {
                if (json.program_message.length >= 1017) {
                    json.program_message = json.program_message.substring(0, 1016);
                }
                json.program_message = cleanControlChars(json.program_message);
                embed.addField('Program Output', `\`\`\`${json.program_message}\`\`\``);
            }
            message.channel.send(embed).then((msg) => {
                if (json.status == 0)
                    msg.react('✅');
                else
                    msg.react('❌');
            });
        });
    });
}

module.exports.help = {
    name:"compile",
    description:"compiles given code"
}