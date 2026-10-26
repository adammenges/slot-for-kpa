package fyi.slot.konkr

import java.net.URLEncoder

/** Content identity only. Renamed games work; hacks and ambiguous CRCs keep their labels. */
internal object SnesArtMatcher {
    fun match(catalog: OfficialArtMatcher.Catalog, fingerprint: CartStudioCatalog.Fingerprint): String? {
        if (catalog.platform != "SNES" || fingerprint.bytes <= 0) return null
        return catalog.entries.filter { it.crc == fingerprint.hex && it.size == fingerprint.bytes }
            .map { it.name }.distinct().singleOrNull()
    }

    fun url(name: String): String {
        require(name.isNotBlank() && name.length <= 512 && name.none { it.isISOControl() })
        val safe = name.map { if (it in "&*/:`<>?\\|\"") '_' else it }.joinToString("")
        val encoded = URLEncoder.encode("$safe.png", "UTF-8").replace("+", "%20")
        return "https://raw.githubusercontent.com/libretro-thumbnails/" +
            "Nintendo_-_Super_Nintendo_Entertainment_System/master/Named_Boxarts/$encoded"
    }
}
