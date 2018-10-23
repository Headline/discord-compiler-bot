const DiscordMessageMenu = require ('./../menus');

module.exports.run = async (client, message, args, prefix) => {

    let menu = new DiscordMessageMenu(message, 'Discord Compiler Bot menu:', 0xFF00FF);
    let items = ['item1', 'item2', 'item3', 'item4', 'item5', 'item6', 'item7'];
    menu.buildMenu(items);
    menu.displayPage(0);
}

module.exports.help = {
    name:"compilers",
    description:"displays all compilers"
}