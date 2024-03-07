export function rotateRecord(record: Record<number, string>): Record<number, string> {
    const keys = Object.keys(record).map(Number).sort((a, b) => a - b);
    const lastKey = keys[keys.length - 1];
    const rotatedRecord: Record<number, string> = {};

    keys.forEach((key, index) => {
        // If it's the last element, map it to the first key's value
        if (index === keys.length - 1) {
            rotatedRecord[0] = record[lastKey];
        } else {
            // Otherwise, map current key to the next key's value
            rotatedRecord[key + 1] = record[key];
        }
    });

    return rotatedRecord;
}


