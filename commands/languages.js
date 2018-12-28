const DiscordMessageMenu = require ('./../menus');

module.exports.run = async (client, message, args, prefix, compilerAPI) => {
    let menu = new DiscordMessageMenu(message, 'Supported Programming Languages:', 0x00FF00, 15);
    menu.buildMenu(compilerAPI.languages);
    menu.displayPage(0);
}

module.exports.help = {
    name:"languages",
    description:"displays all supported languages",
    dev: false
}