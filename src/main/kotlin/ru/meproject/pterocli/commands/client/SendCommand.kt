package ru.meproject.pterocli.commands.client

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.requireObject
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.help
import com.github.ajalt.clikt.parameters.groups.provideDelegate
import com.mattmalec.pterodactyl4j.client.entities.PteroClient
import ru.meproject.pterocli.options.ServerIds

class SendCommand: CliktCommand(
    name= "sendcommand",
    help = "send arbitrary console command to a server"
) {
    private val api by requireObject<PteroClient>()
    private val servers by ServerIds()
    private val serverCommand: String by argument().help("Console command to send")

    override fun run() {
        for (server in servers.ids) {
            api.retrieveServerByIdentifier(server)
                .flatMap { it.sendCommand(serverCommand) }
                .execute()
            echo("Sending console command \"$serverCommand\" to server $server")
        }
    }
}