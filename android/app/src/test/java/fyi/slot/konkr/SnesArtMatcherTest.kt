package fyi.slot.konkr

import org.junit.Assert.*
import org.junit.Test
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

class SnesArtMatcherTest {
    private val rom = ByteArray(32768) { (it * 17).toByte() }
    private fun identity(bytes: ByteArray) = CartStudioCatalog.fingerprintStream(
        object : ByteArrayInputStream(bytes) {
            override fun read(b: ByteArray, off: Int, len: Int) = super.read(b,off,minOf(len,7))
        }, true)
    private fun catalog(name: String, crc: String, size: Long, extra: String = "") =
        OfficialArtMatcher.Catalog.parse("""
            clrmamepro (
                name "SNES"
            )
            game (
                name "$name"
                rom ( name "$name.sfc" size $size crc $crc )
            )
        """.trimIndent() + "\n" + extra, "SNES")

    @Test fun rawHeaderedAndZipRomHaveOneContentIdentity() {
        val raw=identity(rom)
        val headered=ByteArray(512) { 93 } + rom
        val smc=identity(headered)
        val out=ByteArrayOutputStream()
        ZipOutputStream(out).use { zip ->
            zip.putNextEntry(ZipEntry("renamed.smc")); zip.write(headered); zip.closeEntry()
        }
        val zipped=RomSource.useRom(ByteArrayInputStream(out.toByteArray()),"renamed.zip") { input,_ ->
            CartStudioCatalog.fingerprintStream(input,true)
        }
        for (actual in listOf(smc,zipped)) {
            assertEquals(raw.crc,actual.crc); assertEquals(32768L,actual.bytes)
            assertArrayEquals(raw.head,actual.head)
        }
        val gb=CartStudioCatalog.fingerprintStream(ByteArrayInputStream(headered))
        assertEquals(33280L,gb.bytes)
        assertNotEquals(raw.crc,gb.crc)
    }

    @Test fun artworkRequiresUniqueCrcAndSizeAndNeverUsesRenamedTitleGuessing() {
        val fingerprint=identity(rom)
        val dat=catalog("Example (USA)",fingerprint.hex,fingerprint.bytes)
        assertEquals("Example (USA)",SnesArtMatcher.match(dat,fingerprint))
        assertNull(SnesArtMatcher.match(dat,fingerprint.copy(bytes=123)))
        assertNull(SnesArtMatcher.match(dat,identity(rom.copyOf().also { it[100]=8 })))
        val ambiguous=catalog("Example (USA)",fingerprint.hex,fingerprint.bytes,"""
            game (
                name "Other (Japan)"
                rom ( name "Other.sfc" size ${fingerprint.bytes} crc ${fingerprint.hex} )
            )
        """.trimIndent())
        assertNull(SnesArtMatcher.match(ambiguous,fingerprint))
    }

    @Test fun thumbnailNamesAreEscapedAsOneFixedHostPathSegment() {
        val url=SnesArtMatcher.url("A & B: Test/Name (USA)")
        assertTrue(url.endsWith("Named_Boxarts/A%20_%20B_%20Test_Name%20%28USA%29.png"))
        assertTrue(url.startsWith("https://raw.githubusercontent.com/libretro-thumbnails/"))
    }
}
