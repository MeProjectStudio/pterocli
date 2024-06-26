package ru.meproject.pterocli.options

import com.github.ajalt.clikt.parameters.groups.OptionGroup
import com.github.ajalt.clikt.parameters.options.help
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.required
import com.github.ajalt.clikt.parameters.options.split

class ServerIds: OptionGroup() {
    val ids: List<String> by option("-s", "--servers")
        .split(",")
        .required()
        .help("List of short server IDs to perform command on. First 8 characters of UUID.")
}