/**
 * Country flags mapping for the application
 * Centralized country code to emoji flag definitions
 */

/**
 * Extended mapping of country codes to emoji flags
 */
export const countryFlags: Record<string, string> = {
  // North America
  'US': '🇺🇸', 'CA': '🇨🇦', 'MX': '🇲🇽',
  // Europe
  'GB': '🇬🇧', 'DE': '🇩🇪', 'NL': '🇳🇱', 'FR': '🇫🇷', 'IT': '🇮🇹', 'ES': '🇪🇸',
  'PT': '🇵🇹', 'PL': '🇵🇱', 'RU': '🇷🇺', 'UA': '🇺🇦', 'FI': '🇫🇮', 'SE': '🇸🇪',
  'NO': '🇳🇴', 'DK': '🇩🇰', 'CH': '🇨🇭', 'AT': '🇦🇹', 'BE': '🇧🇪', 'IE': '🇮🇪',
  'CZ': '🇨🇿', 'RO': '🇷🇴', 'HU': '🇭🇺', 'BG': '🇧🇬', 'GR': '🇬🇷', 'LU': '🇱🇺',
  'EE': '🇪🇪', 'LV': '🇱🇻', 'LT': '🇱🇹', 'SK': '🇸🇰', 'SI': '🇸🇮', 'HR': '🇭🇷',
  'RS': '🇷🇸', 'MD': '🇲🇩', 'BY': '🇧🇾', 'IS': '🇮🇸', 'AL': '🇦🇱', 'MK': '🇲🇰',
  'ME': '🇲🇪', 'BA': '🇧🇦', 'XK': '🇽🇰', 'MT': '🇲🇹', 'CY': '🇨🇾',
  // Asia
  'JP': '🇯🇵', 'SG': '🇸🇬', 'HK': '🇭🇰', 'KR': '🇰🇷', 'TW': '🇹🇼', 'CN': '🇨🇳',
  'IN': '🇮🇳', 'ID': '🇮🇩', 'TH': '🇹🇭', 'VN': '🇻🇳', 'MY': '🇲🇾', 'PH': '🇵🇭',
  'KZ': '🇰🇿', 'GE': '🇬🇪', 'AM': '🇦🇲', 'AZ': '🇦🇿', 'UZ': '🇺🇿', 'KG': '🇰🇬',
  'MN': '🇲🇳', 'NP': '🇳🇵', 'BD': '🇧🇩', 'LK': '🇱🇰', 'PK': '🇵🇰', 'MM': '🇲🇲',
  'KH': '🇰🇭', 'LA': '🇱🇦', 'BN': '🇧🇳', 'MO': '🇲🇴',
  // Middle East
  'AE': '🇦🇪', 'IL': '🇮🇱', 'TR': '🇹🇷', 'SA': '🇸🇦', 'QA': '🇶🇦', 'KW': '🇰🇼',
  'BH': '🇧🇭', 'OM': '🇴🇲', 'JO': '🇯🇴', 'LB': '🇱🇧', 'IQ': '🇮🇶', 'IR': '🇮🇷',
  // Oceania
  'AU': '🇦🇺', 'NZ': '🇳🇿', 'FJ': '🇫🇯',
  // South America
  'BR': '🇧🇷', 'AR': '🇦🇷', 'CL': '🇨🇱', 'CO': '🇨🇴', 'PE': '🇵🇪', 'VE': '🇻🇪',
  'EC': '🇪🇨', 'UY': '🇺🇾', 'PY': '🇵🇾', 'BO': '🇧🇴',
  // Africa
  'ZA': '🇿🇦', 'EG': '🇪🇬', 'NG': '🇳🇬', 'KE': '🇰🇪', 'MA': '🇲🇦', 'TN': '🇹🇳',
  'GH': '🇬🇭', 'TZ': '🇹🇿', 'UG': '🇺🇬', 'ET': '🇪🇹',
};

/**
 * Get country flag emoji by country code
 * @param countryCode - ISO 3166-1 alpha-2 country code (case-insensitive)
 * @returns Flag emoji or default globe emoji
 */
export function getCountryFlag(countryCode: string | null | undefined): string {
  if (!countryCode) return '🌐';
  return countryFlags[countryCode.toUpperCase()] || '🌐';
}

/**
 * Country names mapping
 */
export const countryNames: Record<string, string> = {
  // North America
  'US': 'United States', 'CA': 'Canada', 'MX': 'Mexico',
  // Europe
  'GB': 'United Kingdom', 'DE': 'Germany', 'NL': 'Netherlands', 'FR': 'France', 
  'IT': 'Italy', 'ES': 'Spain', 'PT': 'Portugal', 'PL': 'Poland', 'RU': 'Russia', 
  'UA': 'Ukraine', 'FI': 'Finland', 'SE': 'Sweden', 'NO': 'Norway', 'DK': 'Denmark', 
  'CH': 'Switzerland', 'AT': 'Austria', 'BE': 'Belgium', 'IE': 'Ireland',
  'CZ': 'Czech Republic', 'RO': 'Romania', 'HU': 'Hungary', 'BG': 'Bulgaria', 
  'GR': 'Greece', 'LU': 'Luxembourg', 'EE': 'Estonia', 'LV': 'Latvia', 
  'LT': 'Lithuania', 'SK': 'Slovakia', 'SI': 'Slovenia', 'HR': 'Croatia',
  'RS': 'Serbia', 'MD': 'Moldova', 'BY': 'Belarus', 'IS': 'Iceland', 
  'AL': 'Albania', 'MK': 'North Macedonia', 'ME': 'Montenegro', 
  'BA': 'Bosnia and Herzegovina', 'XK': 'Kosovo', 'MT': 'Malta', 'CY': 'Cyprus',
  // Asia
  'JP': 'Japan', 'SG': 'Singapore', 'HK': 'Hong Kong', 'KR': 'South Korea', 
  'TW': 'Taiwan', 'CN': 'China', 'IN': 'India', 'ID': 'Indonesia', 
  'TH': 'Thailand', 'VN': 'Vietnam', 'MY': 'Malaysia', 'PH': 'Philippines',
  'KZ': 'Kazakhstan', 'GE': 'Georgia', 'AM': 'Armenia', 'AZ': 'Azerbaijan', 
  'UZ': 'Uzbekistan', 'KG': 'Kyrgyzstan', 'MN': 'Mongolia', 'NP': 'Nepal', 
  'BD': 'Bangladesh', 'LK': 'Sri Lanka', 'PK': 'Pakistan', 'MM': 'Myanmar',
  'KH': 'Cambodia', 'LA': 'Laos', 'BN': 'Brunei', 'MO': 'Macau',
  // Middle East
  'AE': 'UAE', 'IL': 'Israel', 'TR': 'Turkey', 'SA': 'Saudi Arabia', 
  'QA': 'Qatar', 'KW': 'Kuwait', 'BH': 'Bahrain', 'OM': 'Oman', 
  'JO': 'Jordan', 'LB': 'Lebanon', 'IQ': 'Iraq', 'IR': 'Iran',
  // Oceania
  'AU': 'Australia', 'NZ': 'New Zealand', 'FJ': 'Fiji',
  // South America
  'BR': 'Brazil', 'AR': 'Argentina', 'CL': 'Chile', 'CO': 'Colombia', 
  'PE': 'Peru', 'VE': 'Venezuela', 'EC': 'Ecuador', 'UY': 'Uruguay', 
  'PY': 'Paraguay', 'BO': 'Bolivia',
  // Africa
  'ZA': 'South Africa', 'EG': 'Egypt', 'NG': 'Nigeria', 'KE': 'Kenya', 
  'MA': 'Morocco', 'TN': 'Tunisia', 'GH': 'Ghana', 'TZ': 'Tanzania', 
  'UG': 'Uganda', 'ET': 'Ethiopia',
};

/**
 * Get country name by country code
 * @param countryCode - ISO 3166-1 alpha-2 country code (case-insensitive)
 * @returns Country name or 'Unknown'
 */
export function getCountryName(countryCode: string | null | undefined): string {
  if (!countryCode) return 'Unknown';
  return countryNames[countryCode.toUpperCase()] || countryCode.toUpperCase();
}

// ============================================================================
// Server-based country detection
// ============================================================================

/**
 * TLD to country code mapping
 */
const TLD_COUNTRIES: Record<string, string> = {
  // Europe
  'ru': 'RU', 'de': 'DE', 'nl': 'NL', 'uk': 'GB', 'fr': 'FR', 'it': 'IT',
  'es': 'ES', 'pt': 'PT', 'pl': 'PL', 'ua': 'UA', 'fi': 'FI', 'se': 'SE',
  'no': 'NO', 'dk': 'DK', 'ch': 'CH', 'at': 'AT', 'be': 'BE', 'ie': 'IE',
  'cz': 'CZ', 'ro': 'RO', 'hu': 'HU', 'bg': 'BG', 'gr': 'GR', 'lu': 'LU',
  'ee': 'EE', 'lv': 'LV', 'lt': 'LT', 'sk': 'SK', 'si': 'SI', 'hr': 'HR',
  'rs': 'RS', 'md': 'MD', 'by': 'BY', 'is': 'IS', 'al': 'AL', 'mk': 'MK',
  'me': 'ME', 'ba': 'BA', 'mt': 'MT', 'cy': 'CY',
  // Asia
  'jp': 'JP', 'sg': 'SG', 'hk': 'HK', 'kr': 'KR', 'tw': 'TW', 'cn': 'CN',
  'in': 'IN', 'id': 'ID', 'th': 'TH', 'vn': 'VN', 'my': 'MY', 'ph': 'PH',
  'kz': 'KZ', 'ge': 'GE', 'am': 'AM', 'az': 'AZ', 'uz': 'UZ', 'kg': 'KG',
  'mn': 'MN', 'np': 'NP', 'bd': 'BD', 'lk': 'LK', 'pk': 'PK', 'mm': 'MM',
  'kh': 'KH', 'la': 'LA', 'bn': 'BN', 'mo': 'MO',
  // Middle East
  'ae': 'AE', 'il': 'IL', 'tr': 'TR', 'sa': 'SA', 'qa': 'QA', 'kw': 'KW',
  'bh': 'BH', 'om': 'OM', 'jo': 'JO', 'lb': 'LB', 'iq': 'IQ', 'ir': 'IR',
  // Americas
  'us': 'US', 'ca': 'CA', 'mx': 'MX', 'br': 'BR', 'ar': 'AR', 'cl': 'CL',
  'co': 'CO', 'pe': 'PE', 've': 'VE', 'ec': 'EC', 'uy': 'UY', 'py': 'PY', 'bo': 'BO',
  // Oceania
  'au': 'AU', 'nz': 'NZ', 'fj': 'FJ',
  // Africa
  'za': 'ZA', 'eg': 'EG', 'ng': 'NG', 'ke': 'KE', 'ma': 'MA', 'tn': 'TN',
  'gh': 'GH', 'tz': 'TZ', 'ug': 'UG', 'et': 'ET',
};

/**
 * Known hosting providers and their typical locations
 */
const PROVIDER_COUNTRIES: Record<string, string> = {
  // US providers
  'digitalocean': 'US',
  'vultr': 'US',
  'linode': 'US',
  'aws': 'US',
  'amazon': 'US',
  'cloudflare': 'US',
  'google': 'US',
  'gcp': 'US',
  'azure': 'US',
  'microsoft': 'US',
  'oracle': 'US',
  'rackspace': 'US',
  'heroku': 'US',
  'vercel': 'US',
  'netlify': 'US',
  'fastly': 'US',
  'akamai': 'US',
  // European providers
  'hetzner': 'DE',
  'contabo': 'DE',
  'ovh': 'FR',
  'scaleway': 'FR',
  'online.net': 'FR',
  'leaseweb': 'NL',
  'time4vps': 'LT',
  'hostinger': 'LT',
  'ionos': 'DE',
  'strato': 'DE',
  'webgo': 'DE',
  'netcup': 'DE',
  'aruba': 'IT',
  'infomaniak': 'CH',
  'gandi': 'FR',
  // Asian providers
  'alibaba': 'CN',
  'aliyun': 'CN',
  'tencent': 'CN',
  'sakura': 'JP',
  'conoha': 'JP',
  'idcf': 'JP',
  'naver': 'KR',
  'kakao': 'KR',
  'nhn': 'KR',
  // Russian providers
  'selectel': 'RU',
  'timeweb': 'RU',
  'reg.ru': 'RU',
  'beget': 'RU',
  'firstbyte': 'RU',
  'ruvds': 'RU',
  'vdsina': 'RU',
  'firstvds': 'RU',
  'mchost': 'RU',
  'hostland': 'RU',
  'sprinthost': 'RU',
  'jino': 'RU',
  'majordomo': 'RU',
  'nic.ru': 'RU',
  'masterhost': 'RU',
  'ihor': 'RU',
  'fornex': 'RU',
  'justhost': 'RU',
  'sweb': 'RU',
  'fozzy': 'UA',
  'ukraine': 'UA',
};

/**
 * Common location keywords in hostnames
 */
const LOCATION_KEYWORDS: Record<string, string> = {
  // Cities/Regions -> Country codes
  'moscow': 'RU', 'msk': 'RU', 'spb': 'RU', 'petersburg': 'RU',
  'frankfurt': 'DE', 'fra': 'DE', 'berlin': 'DE', 'munich': 'DE',
  'amsterdam': 'NL', 'ams': 'NL', 'rotterdam': 'NL',
  'london': 'GB', 'lon': 'GB', 'manchester': 'GB',
  'paris': 'FR', 'par': 'FR', 'marseille': 'FR',
  'tokyo': 'JP', 'tyo': 'JP', 'osaka': 'JP',
  'singapore': 'SG', 'sgp': 'SG', 'sin': 'SG',
  'hongkong': 'HK', 'hkg': 'HK',
  'seoul': 'KR', 'icn': 'KR',
  'taipei': 'TW', 'tpe': 'TW',
  'sydney': 'AU', 'syd': 'AU', 'melbourne': 'AU',
  'toronto': 'CA', 'yyz': 'CA', 'vancouver': 'CA', 'yvr': 'CA', 'montreal': 'CA',
  'newyork': 'US', 'nyc': 'US', 'losangeles': 'US', 'lax': 'US', 
  'chicago': 'US', 'ord': 'US', 'dallas': 'US', 'dfw': 'US',
  'miami': 'US', 'mia': 'US', 'seattle': 'US', 'sea': 'US',
  'atlanta': 'US', 'atl': 'US', 'denver': 'US', 'den': 'US',
  'phoenix': 'US', 'phx': 'US', 'sanfrancisco': 'US', 'sfo': 'US',
  'saopaulo': 'BR', 'gru': 'BR', 'rio': 'BR',
  'mumbai': 'IN', 'bom': 'IN', 'delhi': 'IN', 'bangalore': 'IN',
  'dubai': 'AE', 'dxb': 'AE',
  'istanbul': 'TR', 'ist': 'TR', 'ankara': 'TR',
  'warsaw': 'PL', 'waw': 'PL', 'krakow': 'PL',
  'prague': 'CZ', 'prg': 'CZ',
  'vienna': 'AT', 'vie': 'AT',
  'zurich': 'CH', 'zrh': 'CH', 'geneva': 'CH',
  'stockholm': 'SE', 'arn': 'SE',
  'helsinki': 'FI', 'hel': 'FI',
  'oslo': 'NO', 'osl': 'NO',
  'copenhagen': 'DK', 'cph': 'DK',
  'dublin': 'IE', 'dub': 'IE',
  'madrid': 'ES', 'mad': 'ES', 'barcelona': 'ES', 'bcn': 'ES',
  'lisbon': 'PT', 'lis': 'PT',
  'rome': 'IT', 'fco': 'IT', 'milan': 'IT', 'mxp': 'IT',
  'brussels': 'BE', 'bru': 'BE',
  'bucharest': 'RO', 'otp': 'RO',
  'budapest': 'HU', 'bud': 'HU',
  'sofia': 'BG', 'sof': 'BG',
  'athens': 'GR', 'ath': 'GR',
  'kyiv': 'UA', 'kiev': 'UA', 'kbp': 'UA', 'odessa': 'UA',
  'minsk': 'BY', 'msq': 'BY',
  'almaty': 'KZ', 'ala': 'KZ', 'astana': 'KZ',
  'tashkent': 'UZ', 'tas': 'UZ',
  'tbilisi': 'GE', 'tbs': 'GE',
  'yerevan': 'AM', 'evn': 'AM',
  'baku': 'AZ', 'gyd': 'AZ',
  // Country name keywords
  'russia': 'RU', 'russian': 'RU',
  'germany': 'DE', 'german': 'DE', 'deutsch': 'DE',
  'netherlands': 'NL', 'dutch': 'NL', 'holland': 'NL',
  'france': 'FR', 'french': 'FR',
  'japan': 'JP', 'japanese': 'JP',
  'korea': 'KR', 'korean': 'KR',
  'china': 'CN', 'chinese': 'CN',
  'india': 'IN', 'indian': 'IN',
  'brazil': 'BR', 'brazilian': 'BR',
  'canada': 'CA', 'canadian': 'CA',
  'australia': 'AU', 'australian': 'AU',
  'ukraine': 'UA', 'ukrainian': 'UA',
  'poland': 'PL', 'polish': 'PL',
  'turkey': 'TR', 'turkish': 'TR',
  'finland': 'FI', 'finnish': 'FI',
  'sweden': 'SE', 'swedish': 'SE',
  'norway': 'NO', 'norwegian': 'NO',
  'denmark': 'DK', 'danish': 'DK',
  'switzerland': 'CH', 'swiss': 'CH',
  'austria': 'AT', 'austrian': 'AT',
  'belgium': 'BE', 'belgian': 'BE',
  'ireland': 'IE', 'irish': 'IE',
  'spain': 'ES', 'spanish': 'ES',
  'portugal': 'PT', 'portuguese': 'PT',
  'italy': 'IT', 'italian': 'IT',
  'greece': 'GR', 'greek': 'GR',
  'czech': 'CZ',
  'romania': 'RO', 'romanian': 'RO',
  'hungary': 'HU', 'hungarian': 'HU',
  'bulgaria': 'BG', 'bulgarian': 'BG',
  'serbia': 'RS', 'serbian': 'RS',
  'croatia': 'HR', 'croatian': 'HR',
  'slovenia': 'SI', 'slovenian': 'SI',
  'slovakia': 'SK', 'slovak': 'SK',
  'estonia': 'EE', 'estonian': 'EE',
  'latvia': 'LV', 'latvian': 'LV',
  'lithuania': 'LT', 'lithuanian': 'LT',
  'belarus': 'BY', 'belarusian': 'BY',
  'moldova': 'MD', 'moldovan': 'MD',
  'georgia': 'GE', 'georgian': 'GE',
  'armenia': 'AM', 'armenian': 'AM',
  'azerbaijan': 'AZ', 'azerbaijani': 'AZ',
  'kazakhstan': 'KZ', 'kazakh': 'KZ',
  'uzbekistan': 'UZ', 'uzbek': 'UZ',
  'israel': 'IL', 'israeli': 'IL',
  'uae': 'AE', 'emirates': 'AE',
  'singaporean': 'SG',
  'taiwan': 'TW', 'taiwanese': 'TW',
  'vietnam': 'VN', 'vietnamese': 'VN',
  'thailand': 'TH', 'thai': 'TH',
  'malaysia': 'MY', 'malaysian': 'MY',
  'indonesia': 'ID', 'indonesian': 'ID',
  'philippines': 'PH', 'filipino': 'PH',
};

/**
 * Detect country code from server hostname
 * Uses TLD, known provider heuristics, and location keywords
 * @param server - Server hostname or IP address
 * @returns ISO 3166-1 alpha-2 country code or null if unknown
 */
export function detectCountryFromServer(server: string | null | undefined): string | null {
  if (!server) return null;
  
  const hostname = server.toLowerCase().trim();
  
  // Skip IP addresses (can't determine country without GeoIP)
  if (/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$/.test(hostname)) {
    return null;
  }
  
  // Check TLD first (most reliable)
  const parts = hostname.split('.');
  if (parts.length >= 2) {
    const tld = parts[parts.length - 1];
    if (TLD_COUNTRIES[tld]) {
      return TLD_COUNTRIES[tld];
    }
  }
  
  // Check for known providers in hostname
  for (const [provider, country] of Object.entries(PROVIDER_COUNTRIES)) {
    if (hostname.includes(provider)) {
      return country;
    }
  }
  
  // Check for location keywords in hostname (cities, countries)
  // Split hostname into parts for matching
  const hostnameParts = hostname.replace(/[.-]/g, ' ').split(' ');
  for (const part of hostnameParts) {
    if (part.length >= 2 && LOCATION_KEYWORDS[part]) {
      return LOCATION_KEYWORDS[part];
    }
  }
  
  // Also check for keywords as substrings (e.g., "moscow1.example.com")
  for (const [keyword, country] of Object.entries(LOCATION_KEYWORDS)) {
    if (keyword.length >= 4 && hostname.includes(keyword)) {
      return country;
    }
  }
  
  return null;
}

/**
 * Get country flag for a proxy, with fallback to server-based detection
 * @param country - Explicit country code (if available)
 * @param server - Server hostname for fallback detection
 * @returns Flag emoji
 */
export function getProxyFlag(country: string | null | undefined, server?: string): string {
  // First try explicit country
  if (country) {
    return getCountryFlag(country);
  }
  
  // Fallback to server-based detection
  const detected = detectCountryFromServer(server);
  return getCountryFlag(detected);
}

/**
 * Get country name for a proxy, with fallback to server-based detection
 * @param country - Explicit country code (if available)
 * @param server - Server hostname for fallback detection
 * @returns Country name
 */
export function getProxyCountryName(country: string | null | undefined, server?: string): string {
  // First try explicit country
  if (country) {
    return getCountryName(country);
  }
  
  // Fallback to server-based detection
  const detected = detectCountryFromServer(server);
  return getCountryName(detected);
}
