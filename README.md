# TermiBot

## Prerequisites

 * dotnet core: https://dotnet.microsoft.com/download

## Run tests

 * `dotnet test termibot.karma.tests`

 ## Setup & Run

 * If needed add a new bot to your workspace, following Noobot's instructions here: https://github.com/noobot/noobot/wiki/Getting-Started-With-Noobot#get-noobot-up-and-running-quickly
 * Download a copy of `config.default.json` here: https://github.com/noobot/noobot/blob/master/src/Noobot.Console/Configuration/config.default.json
 * Move it to `TermiBot.Console/Configuration/config.json` file and replce `slack:apiToken` with your bot's API token
   * To find a copy of your bot's API token go to https://api.slack.com/apps, select your bot, go to "Install App" and copy the "Bot User OAuth Access Token". It should start with `xoxb`.
 * `dotnet run --project TermiBot.Console/TermiBot.Console.csproj`