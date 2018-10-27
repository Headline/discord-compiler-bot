const Discord = require('discord.js');
const WandBox = require ('../WandBox.js');
const stripAnsi = require('strip-ansi');

// py
const fs = require('fs');
const spawn = require('child_process').spawn;

function cleanControlChars(dirty) {
    return stripAnsi(dirty);
}

module.exports.run = async (client, message, args, prefix, compilerAPI, cmdlist) => {

    let match = message.content.match('```([\\s\\S]*)```');
    if (!match)
    {
        const embed = new Discord.RichEmbed()
        .setTitle('Error:')
        .setColor(0xFF0000)
        .setDescription(`You must attach codeblocks containing code to your message`)
        message.channel.send(embed).then((msg) => {
            let group = [message, msg];
            cmdlist.push(group);            
        });
        return;
    }

    let code = message.content.match('```([\\s\\S]*)```')[1].trim();
    if (code.length <= 0)
    {
        const embed = new Discord.RichEmbed()
        .setTitle('Error:')
        .setColor(0xFF0000)
        .setDescription(`You must actually supply code to compile!`)
        message.channel.send(embed).then((msg) => {
            let group = [message, msg];
            cmdlist.push(group);            
        });
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
        message.channel.send(embed).then((msg) => {
            let group = [message, msg];
            cmdlist.push(group);            
        });
        return;
    }

    let discordLanguages = [ 'asciidoc', 'autohotkey', 'bash', 'coffeescript', 'cpp',
    'cs', 'css', 'diff', 'fix', 'glsl', 'ini', 'java', 'json', 'md', 'ml', 'prolog', 'python',
    'py', 'tex', 'xml', 'xl'];
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

            /* The request failed */
            if (json == null) {
                embed.setColor(0xFF0000);
                embed.setDescription("It appears that a request has failed. It has either timed out or wandbox.org is rejecting requests. Please try again later.");
                message.channel.send(embed).then((msg) => {
                    let group = [message, msg];
                    cmdlist.push(group);            
                });
                return;
            }

            /* We got something back, build embed. */
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

                /* On the deployed instance, we will track usage. This wont
                 * have any effect if this file doesn't exist on your fs */
                let file = '/var/www/html/discord-compiler/graph.py';
                fs.stat(file, (err, stat) => {
                    if (err == null) {
                        const process = spawn('python', file);
                    }
                });

                if (json.status == 0)
                    msg.react('✅');
                else
                    msg.react('❌');

                let group = [message, msg];
                cmdlist.push(group);        
            });
        });
    });
}

module.exports.help = {
    name:"compile",
    description:"compiles given code"
}