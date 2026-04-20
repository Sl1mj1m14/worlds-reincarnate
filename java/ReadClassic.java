import java.io.ByteArrayInputStream;
import java.io.DataInputStream;
import java.io.IOException;

public class ReadClassic {

    final static short STREAM_MAGIC = (short)0xaced;
    final static short STREAM_VERSION = 5;

    final static byte TC_NULL = 0x70;
    final static byte TC_REFERENCE = 0x71;
    final static byte TC_CLASSDESC = 0x72;
    final static byte TC_OBJECT = 0x73;
    final static byte TC_STRING = 0x74;
    final static byte TC_ARRAY = 0x75;
    final static byte TC_CLASS = 0x76;
    final static byte TC_BLOCKDATA = 0x77;
    final static byte TC_ENDBLOCKDATA = 0x78;
    final static byte TC_RESET = 0x79;
    final static byte TC_BLOCKDATALONG = 0x7A;
    final static byte TC_EXCEPTION = 0x7B;
    final static byte TC_LONGSTRING = 0x7C;
    final static byte TC_PROXYCLASSDESC = 0x7D;
    final static byte TC_ENUM = 0x7E;

    final static int baseWireHandle = 0x7E0000;

    final static byte SC_WRITE_METHOD = 0x01; //if SC_SERIALIZABLE
    final static byte SC_BLOCK_DATA = 0x08;    //if SC_EXTERNALIZABLE
    final static byte SC_SERIALIZABLE = 0x02;
    final static byte SC_EXTERNALIZABLE = 0x04;
    final static byte SC_ENUM = 0x10;

    public static void read(byte[] stream) throws IOException {

        ByteArrayInputStream bin = new ByteArrayInputStream(stream);
        DataInputStream din = new DataInputStream(bin);

        int magic = din.readShort();
        if (magic != STREAM_MAGIC) {
            System.out.println("Invalid magic value - unable to deserialize");
            throw new IllegalArgumentException("Invalid magic value");
        }

        int version = din.readUnsignedShort();
		if(version != STREAM_VERSION) {
            System.out.println("Invalid version - unable to deserialize");
			throw new IllegalArgumentException("Invalid version number");
		}
        
    }
}
