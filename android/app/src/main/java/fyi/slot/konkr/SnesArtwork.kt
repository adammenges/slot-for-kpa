package fyi.slot.konkr

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.RectF
import android.os.SystemClock
import android.util.AtomicFile
import android.util.Log
import org.json.JSONObject
import java.io.ByteArrayOutputStream
import java.io.File
import java.net.HttpURLConnection
import java.net.URL

/** Exact No-Intro identity and Libretro box art, independent of Slot's GB art service. */
internal object SnesArtwork {
    private val retry = AutoLabelRetryPolicy()
    private const val DAT_KEY = "@snes-dat"
    private const val MAX_BYTES = 6 * 1024 * 1024
    private val signature = byteArrayOf(-119,80,78,71,13,10,26,10)
    var nextDelayMs = 4_000L
        private set

    private fun pending(roms: List<CartStudioCatalog.Rom>): Boolean {
        val now = SystemClock.elapsedRealtime()
        val keys = roms.map { it.uri }.filter(retry::pending)
        if (keys.isEmpty() || !retry.pending(DAT_KEY)) return false
        nextDelayMs = maxOf(4_000L, retry.waitMs(DAT_KEY,now), retry.nextDelayMs(keys,now)).coerceAtMost(180_000L)
        return true
    }

    fun fill(context: Context, shouldContinue: () -> Boolean, onSaved: (String) -> Unit): Boolean {
        if (!shouldContinue()) return false
        val missing = CartStudioCatalog.games(context).filter {
            it.platform == "SNES" && !CartLabelPaths.file(context.filesDir,it.uri).exists() && retry.pending(it.uri)
        }
        if (missing.isEmpty()) return false
        val now = SystemClock.elapsedRealtime()
        if (!retry.ready(DAT_KEY,now)) return pending(missing)
        val due = missing.filter { retry.ready(it.uri,now) }.take(4)
        if (due.isEmpty()) return pending(missing)
        val dat = OfficialDatSource.load(context,"SNES",shouldContinue)
        if (dat == null) {
            if (shouldContinue()) retry.transient(DAT_KEY,SystemClock.elapsedRealtime())
            return shouldContinue() && pending(missing)
        }
        retry.reset(DAT_KEY)
        var saved = 0
        for (rom in due) {
            if (!shouldContinue() || saved >= 2) break
            try {
                val fingerprint = CartStudioCatalog.identify(context,rom)
                val name = SnesArtMatcher.match(dat,fingerprint)
                if (name == null) { retry.finished(rom.uri); continue }
                val url = SnesArtMatcher.url(name)
                val bytes = fetch(context,url,fingerprint.hex,shouldContinue)
                if (!shouldContinue()) break
                if (bytes == null) { retry.finished(rom.uri); continue } // Verified HTTP 404.
                val label = label(bytes)
                if (!shouldContinue()) break
                val target = CartLabelPaths.file(context.filesDir,rom.uri)
                synchronized(CartStudioBridge.labelWriteLock) {
                    // A custom image saved during acquisition always wins.
                    if (!target.exists() && shouldContinue()) {
                        atomic(target,label)
                        saved++
                        val provenance = JSONObject().put("provider","Libretro thumbnails")
                            .put("url",url).put("title",name).put("crc32",fingerprint.hex)
                            .put("rom_bytes",fingerprint.bytes)
                        runCatching { atomic(File(target.path + ".source.json"),provenance.toString().toByteArray()) }
                        onSaved(rom.uri)
                        Log.i("SlotAutoLabels","SNES box art saved: $name")
                    }
                }
                if (target.exists()) retry.finished(rom.uri)
            } catch (error: Exception) {
                if (shouldContinue()) retry.transient(rom.uri,SystemClock.elapsedRealtime())
                Log.w("SlotAutoLabels","SNES artwork deferred for ${rom.title.take(80)}",error)
            }
        }
        CartStudioCatalog.flush(context)
        return shouldContinue() && pending(missing)
    }

    private fun fetch(context: Context, url: String, crc: String, keepGoing: () -> Boolean): ByteArray? {
        val dir = File(context.cacheDir,"snes-boxart").apply { mkdirs() }
        val cache = File(dir,"$crc.png")
        if (cache.isFile && cache.length() in 8..MAX_BYTES.toLong()) return cache.readBytes()
        val connection = (URL(url).openConnection() as HttpURLConnection).apply {
            connectTimeout = 9_000; readTimeout = 12_000; instanceFollowRedirects = false
        }
        try {
            if (connection.responseCode == 404) return null
            check(connection.responseCode == 200) { "Artwork HTTP ${connection.responseCode}" }
            val bytes = connection.inputStream.use { input ->
                val output = ByteArrayOutputStream(); val buffer = ByteArray(16384)
                while (keepGoing()) {
                    val count = input.read(buffer); if (count < 0) break
                    require(output.size()+count <= MAX_BYTES) { "Artwork exceeds 6 MiB" }
                    output.write(buffer,0,count)
                }
                check(keepGoing()) { "Artwork paused" }; output.toByteArray()
            }
            require(bytes.size >= 8 && bytes.copyOfRange(0,8).contentEquals(signature)) { "Expected PNG artwork" }
            // Validate image bounds before retaining the download.
            bounds(bytes)
            atomic(cache,bytes)
            var total = dir.listFiles().orEmpty().sumOf { it.length() }
            for (file in dir.listFiles().orEmpty().sortedBy { it.lastModified() }) {
                if (total <= 64L*1024*1024) break
                val size = file.length(); if (file.delete()) total -= size
            }
            return bytes
        } finally { connection.disconnect() }
    }

    private fun bounds(bytes: ByteArray) {
        val options = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        BitmapFactory.decodeByteArray(bytes,0,bytes.size,options)
        require(options.outWidth in 1..4096 && options.outHeight in 1..4096) { "Invalid artwork dimensions" }
    }

    private fun label(bytes: ByteArray): ByteArray {
        bounds(bytes)
        val source = requireNotNull(BitmapFactory.decodeByteArray(bytes,0,bytes.size))
        val output = Bitmap.createBitmap(456,276,Bitmap.Config.ARGB_8888)
        try {
            val canvas = Canvas(output); canvas.drawColor(Color.rgb(60,55,69))
            // Keep the whole cover: Japanese vertical boxes must not lose titles.
            val scale = minOf(456f/source.width,276f/source.height)
            val width = source.width*scale; val height = source.height*scale
            canvas.drawBitmap(source,null,RectF((456-width)/2,(276-height)/2,(456+width)/2,(276+height)/2),Paint(Paint.FILTER_BITMAP_FLAG))
            return ByteArrayOutputStream().use { out ->
                check(output.compress(Bitmap.CompressFormat.PNG,100,out)); out.toByteArray()
            }
        } finally { source.recycle(); output.recycle() }
    }

    private fun atomic(file: File, bytes: ByteArray) {
        file.parentFile?.mkdirs()
        val atomic = AtomicFile(file); val stream = atomic.startWrite()
        try { stream.write(bytes); atomic.finishWrite(stream) }
        catch (error: Exception) { atomic.failWrite(stream); throw error }
    }
}
