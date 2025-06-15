// Minecraft color codes
const COLOR_CODES: {[key: string]: string} = {
  '0': '#000000', // Black
  '1': '#0000AA', // Dark Blue
  '2': '#00AA00', // Dark Green
  '3': '#00AAAA', // Dark Aqua
  '4': '#AA0000', // Dark Red
  '5': '#AA00AA', // Dark Purple
  '6': '#FFAA00', // Gold
  '7': '#AAAAAA', // Gray
  '8': '#555555', // Dark Gray
  '9': '#5555FF', // Blue
  a: '#55FF55', // Green
  b: '#55FFFF', // Aqua
  c: '#FF5555', // Red
  d: '#FF55FF', // Light Purple
  e: '#FFFF55', // Yellow
  f: '#FFFFFF', // White
  r: '#FFFFFF', // Reset
};

type FormatCode = 'l' | 'n' | 'o' | 'k' | 'm';
type FormatType =
  | 'bold'
  | 'underline'
  | 'italic'
  | 'obfuscated'
  | 'strikethrough';

// Format codes
const FORMAT_CODES: Record<FormatCode, FormatType> = {
  l: 'bold',
  n: 'underline',
  o: 'italic',
  k: 'obfuscated',
  m: 'strikethrough',
};

export interface MinecraftTextSegment {
  text: string;
  color?: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strikethrough?: boolean;
  obfuscated?: boolean;
}

export const parseMinecraftColors = (text: string): MinecraftTextSegment[] => {
  const segments: MinecraftTextSegment[] = [];
  let currentSegment: MinecraftTextSegment = {text: ''};
  let i = 0;

  while (i < text.length) {
    if (text[i] === '§' && i + 1 < text.length) {
      const code = text[i + 1].toLowerCase();

      // If we have accumulated text, add it as a segment
      if (currentSegment.text) {
        segments.push({...currentSegment});
        currentSegment = {text: ''};
      }

      // Handle color codes
      if (COLOR_CODES[code]) {
        currentSegment.color = COLOR_CODES[code];
      }
      // Handle format codes
      else if (code in FORMAT_CODES) {
        const format = FORMAT_CODES[code as FormatCode];
        currentSegment[format] = true;
      }
      // Reset formatting
      else if (code === 'r') {
        currentSegment = {text: ''};
      }

      i += 2; // Skip the § and the code
    } else {
      currentSegment.text += text[i];
      i++;
    }
  }

  // Add the last segment if it has text
  if (currentSegment.text) {
    segments.push(currentSegment);
  }

  return segments;
};
