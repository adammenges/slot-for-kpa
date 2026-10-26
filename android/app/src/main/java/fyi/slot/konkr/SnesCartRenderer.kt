package fyi.slot.konkr

/** Pure CPU rendering, called on WebView's bridge worker, never the GL thread. */
internal object SnesCartRenderer {
    init { System.loadLibrary("slot_android") }
    external fun render(stem: String, shell: String): ByteArray
}
