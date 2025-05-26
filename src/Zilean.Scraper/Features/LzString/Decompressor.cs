using System.Runtime.InteropServices;

public static class Decompressor
{
    private static readonly byte[] _uriKey =
        Encoding.ASCII.GetBytes("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-$");

    public static bool TryDecompress(ReadOnlySpan<char> input, out string? result)
    {
        result = null;

        ushort[] buffer = new ushort[input.Length];
        int length = 0;

        foreach (var ch in input)
        {
            var c = ch == ' ' ? '+' : ch;
            int index = Array.IndexOf(_uriKey, (byte)c);
            if (index < 0)
                return false;

            buffer[length++] = (ushort)index;
        }

        var compressed = new ReadOnlySpan<ushort>(buffer, 0, length);
        var decoded = DecompressInternal(compressed, 6);
        if (decoded == null)
            return false;

        result = Encoding.Unicode.GetString(MemoryMarshal.AsBytes(decoded.AsSpan()));
        return true;
    }

    private static ushort[]? DecompressInternal(ReadOnlySpan<ushort> input, byte bitsPerChar)
{
    if (input.IsEmpty)
        return Array.Empty<ushort>();

    int index = 1;
    ushort position = (ushort)(1 << (bitsPerChar - 1));
    ushort resetVal = position;
    ushort val = input[0];

    List<ushort[]> dictionary = new(16)
    {
        new ushort[] { 0 },
        new ushort[] { 1 },
        new ushort[] { 2 }
    };

    int numBits = 3;
    int enlargeIn = 4;

    // Read first code (2 bits)
    int cc = 0;
    int power = 1;
    for (int i = 0; i < 2; i++)
    {
        bool bit = (val & position) != 0;
        position >>= 1;

        if (position == 0 && index < input.Length)
        {
            position = resetVal;
            val = input[index++];
        }

        if (bit)
            cc |= power;

        power <<= 1;
    }

    if (cc == 2)
        return Array.Empty<ushort>();

    ushort firstChar;
    {
        int bitsToRead = cc == 0 ? 8 : 16;
        int raw = 0;
        power = 1;
        for (int i = 0; i < bitsToRead; i++)
        {
            bool bit = (val & position) != 0;
            position >>= 1;

            if (position == 0 && index < input.Length)
            {
                position = resetVal;
                val = input[index++];
            }

            if (bit)
                raw |= power;

            power <<= 1;
        }

        firstChar = (ushort)raw;
    }

    List<ushort> w = new() { firstChar };
    List<ushort> result = new() { firstChar };
    dictionary.Add(new[] { firstChar });

    while (index <= input.Length)
    {
        int code = 0;
        power = 1;

        for (int i = 0; i < numBits; i++)
        {
            bool bit = (val & position) != 0;
            position >>= 1;

            if (position == 0 && index < input.Length)
            {
                position = resetVal;
                val = input[index++];
            }

            if (bit)
                code |= power;

            power <<= 1;
        }

        if (code == 0 || code == 1)
        {
            int bitsToRead = code == 0 ? 8 : 16;
            int raw = 0;
            power = 1;
            for (int i = 0; i < bitsToRead; i++)
            {
                bool bit = (val & position) != 0;
                position >>= 1;

                if (position == 0 && index < input.Length)
                {
                    position = resetVal;
                    val = input[index++];
                }

                if (bit)
                    raw |= power;

                power <<= 1;
            }

            ushort c = (ushort)raw;
            dictionary.Add(new[] { c });
            code = dictionary.Count - 1;
            enlargeIn--;
        }
        else if (code == 2)
        {
            break;
        }

        if (enlargeIn == 0)
        {
            enlargeIn = 1 << numBits;
            numBits++;
        }

        ushort[] entry;
        if (code < dictionary.Count)
        {
            entry = dictionary[code];
        }
        else if (code == dictionary.Count)
        {
            entry = w.Concat(new[] { w[0] }).ToArray(); // safe fallback
        }
        else
        {
            return null;
        }

        result.AddRange(entry);

        var newEntry = new ushort[w.Count + 1];
        w.CopyTo(newEntry, 0);
        newEntry[w.Count] = entry[0];
        dictionary.Add(newEntry);

        w = new(entry);

        enlargeIn--;
        if (enlargeIn == 0)
        {
            enlargeIn = 1 << numBits;
            numBits++;
        }
    }

    return result.ToArray();
}
}
