// Deterministic faker-style data generator for the demo
// grid. Zero dependencies — uses static arrays + a
// simple integer hash for reproducible pseudo-random
// selection.

// ── hash ──────────────────────────────────────────────

/// splitmix64 finalizer — maps sequential u64 to
/// well-distributed u64.
fn hash_row(row: u64) -> u64 {
    let mut x = row.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Hash with a per-field salt to decorrelate picks.
fn hash_field(row: u64, field: u64) -> u64 {
    hash_row(row.wrapping_add(field.wrapping_mul(0x517cc1b727220a95)))
}

/// Pick an element from a static slice by hashed index.
fn pick<'a>(arr: &'a [&str], row: u64, field: u64) -> &'a str {
    arr[hash_field(row, field) as usize % arr.len()]
}

// ── base static data ─────────────────────────────────

const FIRST_NAMES: &[&str] = &[
    "Alice",
    "Bob",
    "Charlie",
    "Diana",
    "Edward",
    "Fatima",
    "George",
    "Hannah",
    "Ivan",
    "Julia",
    "Kenji",
    "Laura",
    "Miguel",
    "Nadia",
    "Oliver",
    "Priya",
    "Quentin",
    "Rachel",
    "Sanjay",
    "Tara",
    "Uma",
    "Victor",
    "Wendy",
    "Xavier",
    "Yuki",
    "Zara",
    "Aaron",
    "Beatrice",
    "Carlos",
    "Delilah",
    "Ethan",
    "Fiona",
    "Gabriel",
    "Helena",
    "Isaac",
    "Jasmine",
    "Kevin",
    "Lily",
    "Marco",
    "Nina",
    "Oscar",
    "Penelope",
    "Rafael",
    "Sofia",
    "Thomas",
    "Ursula",
    "Vincent",
    "Whitney",
    "Xander",
    "Yvette",
    "Zachary",
    "Amelia",
    "Benjamin",
    "Clara",
    "David",
    "Elena",
    "Felix",
    "Grace",
    "Henry",
    "Iris",
    "James",
    "Katherine",
    "Leo",
    "Maya",
    "Nathan",
    "Olivia",
    "Patrick",
    "Quinn",
    "Rosa",
    "Samuel",
    "Tiffany",
    "Ulrich",
    "Valentina",
    "Walter",
    "Ximena",
    "Yolanda",
    "Zane",
    "Aria",
    "Brandon",
    "Chloe",
    "Daniel",
    "Emma",
    "Frank",
    "Giselle",
    "Hugo",
    "Ingrid",
    "Joel",
    "Kira",
    "Lucas",
    "Mia",
    "Noah",
    "Paige",
    "Remy",
    "Stella",
    "Tyler",
    "Violet",
    "Wesley",
    "Yara",
    "Aiden",
    "Bianca",
    "Caleb",
    "Daphne",
];

const LAST_NAMES: &[&str] = &[
    "Smith",
    "Johnson",
    "Williams",
    "Brown",
    "Jones",
    "Garcia",
    "Miller",
    "Davis",
    "Rodriguez",
    "Martinez",
    "Anderson",
    "Taylor",
    "Thomas",
    "Hernandez",
    "Moore",
    "Martin",
    "Jackson",
    "Thompson",
    "White",
    "Lopez",
    "Lee",
    "Harris",
    "Clark",
    "Lewis",
    "Robinson",
    "Walker",
    "Young",
    "Allen",
    "King",
    "Wright",
    "Scott",
    "Torres",
    "Nguyen",
    "Hill",
    "Flores",
    "Green",
    "Adams",
    "Nelson",
    "Baker",
    "Hall",
    "Rivera",
    "Campbell",
    "Mitchell",
    "Carter",
    "Roberts",
    "Gomez",
    "Phillips",
    "Evans",
    "Turner",
    "Diaz",
    "Parker",
    "Cruz",
    "Edwards",
    "Collins",
    "Reyes",
    "Stewart",
    "Morris",
    "Morales",
    "Murphy",
    "Cook",
    "Rogers",
    "Gutierrez",
    "Ortiz",
    "Morgan",
    "Cooper",
    "Chen",
    "Peterson",
    "Bailey",
    "Reed",
    "Kelly",
    "Howard",
    "Ramos",
    "Kim",
    "Cox",
    "Ward",
    "Richardson",
    "Watson",
    "Brooks",
    "Chavez",
    "Wood",
    "James",
    "Bennett",
    "Gray",
    "Mendoza",
    "Ruiz",
    "Hughes",
    "Price",
    "Alvarez",
    "Castillo",
    "Sanders",
    "Patel",
    "Myers",
    "Long",
    "Ross",
    "Foster",
    "Jimenez",
    "Powell",
    "Jenkins",
    "Perry",
    "Russell",
    "Sullivan",
    "Bell",
    "Coleman",
    "Butler",
    "Tanaka",
    "Muller",
    "Fischer",
    "Weber",
    "Schneider",
    "Johansson",
    "Larsson",
    "Dubois",
    "Laurent",
    "Bernard",
    "Rossi",
    "Bianchi",
    "Colombo",
    "Ivanov",
    "Petrov",
    "Nakamura",
    "Yamamoto",
    "Sato",
    "Takahashi",
];

const DEPARTMENTS: &[&str] = &[
    "Engineering",
    "Marketing",
    "Sales",
    "Human Resources",
    "Finance",
    "Operations",
    "Legal",
    "Customer Success",
    "Product",
    "Design",
    "Data Science",
    "Infrastructure",
];

/// (title, salary_min, salary_max)
const ROLES: &[(&str, u32, u32)] = &[
    ("CEO", 250_000, 350_000),
    ("CTO", 220_000, 320_000),
    ("CFO", 210_000, 300_000),
    ("VP of Engineering", 180_000, 260_000),
    ("VP of Sales", 175_000, 250_000),
    ("Senior Software Engineer", 140_000, 200_000),
    ("Engineering Manager", 150_000, 210_000),
    ("Director of Marketing", 130_000, 190_000),
    ("Principal Engineer", 160_000, 220_000),
    ("Senior Product Manager", 135_000, 195_000),
    ("Software Engineer", 90_000, 140_000),
    ("Product Manager", 95_000, 145_000),
    ("Data Analyst", 80_000, 120_000),
    ("UX Designer", 85_000, 130_000),
    ("Marketing Manager", 80_000, 125_000),
    ("Account Executive", 75_000, 130_000),
    ("Junior Developer", 55_000, 80_000),
    ("Sales Associate", 45_000, 70_000),
    ("Marketing Coordinator", 48_000, 72_000),
    ("Support Specialist", 42_000, 65_000),
    ("QA Analyst", 50_000, 78_000),
];

// ── extra column arrays ──────────────────────────────

const CITIES: &[&str] = &[
    "New York",
    "London",
    "Paris",
    "Tokyo",
    "Berlin",
    "Sydney",
    "Toronto",
    "Mumbai",
    "Singapore",
    "Seoul",
    "Amsterdam",
    "Dubai",
    "Stockholm",
    "Barcelona",
    "Chicago",
    "San Francisco",
    "Austin",
    "Denver",
    "Seattle",
    "Portland",
];

/// (code, display_name) — codes are ISO 3166-1 alpha-2.
/// Flag images come from `rs_grid_flags`.
const COUNTRIES: &[(&str, &str)] = &[
    ("US", "United States"),
    ("GB", "United Kingdom"),
    ("FR", "France"),
    ("DE", "Germany"),
    ("JP", "Japan"),
    ("AU", "Australia"),
    ("CA", "Canada"),
    ("IN", "India"),
    ("BR", "Brazil"),
    ("SG", "Singapore"),
    ("KR", "South Korea"),
    ("NL", "Netherlands"),
    ("AE", "UAE"),
    ("SE", "Sweden"),
    ("ES", "Spain"),
    ("IT", "Italy"),
    ("MX", "Mexico"),
    ("CN", "China"),
    ("RU", "Russia"),
    ("ZA", "South Africa"),
    ("NO", "Norway"),
    ("DK", "Denmark"),
    ("FI", "Finland"),
    ("PL", "Poland"),
    ("PT", "Portugal"),
    ("CH", "Switzerland"),
    ("AT", "Austria"),
    ("BE", "Belgium"),
    ("IE", "Ireland"),
    ("NZ", "New Zealand"),
    ("AR", "Argentina"),
    ("CL", "Chile"),
    ("CO", "Colombia"),
    ("IL", "Israel"),
    ("TH", "Thailand"),
    ("MY", "Malaysia"),
    ("PH", "Philippines"),
    ("ID", "Indonesia"),
    ("VN", "Vietnam"),
    ("EG", "Egypt"),
    ("NG", "Nigeria"),
    ("KE", "Kenya"),
    ("GH", "Ghana"),
    ("SA", "Saudi Arabia"),
    ("TR", "Turkey"),
    ("GR", "Greece"),
    ("CZ", "Czech Republic"),
    ("RO", "Romania"),
    ("HU", "Hungary"),
    ("UA", "Ukraine"),
];

const STATES_US: &[&str] = &[
    "CA", "NY", "TX", "WA", "IL", "FL", "MA", "CO", "OR", "GA", "NC", "PA",
    "OH", "AZ", "VA",
];

const GENDERS: &[&str] = &["Male", "Female"];

const TEAMS: &[&str] = &[
    "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta",
    "Iota", "Kappa",
];

const OFFICES: &[&str] = &[
    "HQ",
    "West Campus",
    "East Tower",
    "South Wing",
    "Remote",
    "Satellite",
    "Downtown",
    "Tech Park",
];

const EMP_TYPES: &[&str] = &["Full-time", "Part-time", "Contract", "Intern"];

const SENIORITY_LEVELS: &[&str] =
    &["Junior", "Mid", "Senior", "Staff", "Principal"];

const SKILLS: &[&str] = &[
    "Rust",
    "TypeScript",
    "Python",
    "Go",
    "Java",
    "C++",
    "React",
    "Vue",
    "Angular",
    "Node.js",
    "PostgreSQL",
    "Redis",
    "Docker",
    "Kubernetes",
    "AWS",
    "GCP",
    "Azure",
    "GraphQL",
    "REST",
    "gRPC",
];

const CERTS: &[&str] = &[
    "AWS Solutions Architect",
    "PMP",
    "Scrum Master",
    "CPA",
    "CISSP",
    "Google Cloud Pro",
    "Six Sigma Green Belt",
    "ITIL v4",
    "PHR",
    "CFA",
];

const SPOKEN_LANGS: &[&str] = &[
    "English",
    "Spanish",
    "French",
    "German",
    "Japanese",
    "Mandarin",
    "Portuguese",
    "Korean",
    "Hindi",
    "Arabic",
];

const EDU_LEVELS: &[&str] = &[
    "Bachelor's",
    "Master's",
    "PhD",
    "Associate's",
    "High School",
];

const UNIVERSITIES: &[&str] = &[
    "MIT",
    "Stanford",
    "Harvard",
    "Oxford",
    "Cambridge",
    "ETH Zurich",
    "UC Berkeley",
    "Caltech",
    "Princeton",
    "Yale",
    "Columbia",
    "Carnegie Mellon",
    "Georgia Tech",
    "U Michigan",
    "UCLA",
];

const BENEFITS: &[&str] = &["Platinum", "Gold", "Silver", "Bronze", "Basic"];

const LAPTOPS: &[&str] = &[
    "MacBook Pro 16",
    "MacBook Pro 14",
    "ThinkPad X1",
    "Dell XPS 15",
    "Surface Pro",
    "Framework 16",
];

const OSES: &[&str] = &["macOS", "Windows", "Linux", "ChromeOS"];

const TIMEZONES: &[&str] = &[
    "UTC-8", "UTC-5", "UTC+0", "UTC+1", "UTC+2", "UTC+5:30", "UTC+8", "UTC+9",
];

const PREV_COMPANIES: &[&str] = &[
    "Google",
    "Amazon",
    "Microsoft",
    "Apple",
    "Meta",
    "Netflix",
    "Uber",
    "Stripe",
    "Shopify",
    "Salesforce",
    "Oracle",
    "IBM",
    "Intel",
    "Cisco",
    "Adobe",
];

const REFERRAL_SOURCES: &[&str] = &[
    "LinkedIn",
    "Employee Referral",
    "Job Board",
    "Recruiter",
    "University",
    "Direct Application",
];

const IDES: &[&str] = &["VS Code", "IntelliJ", "Vim", "Neovim", "Emacs", "Zed"];

const PROG_LANGS: &[&str] = &[
    "Rust",
    "Python",
    "TypeScript",
    "Go",
    "Java",
    "C#",
    "Kotlin",
    "Swift",
    "Ruby",
    "Elixir",
];

const BLOOD_TYPES: &[&str] =
    &["A+", "A-", "B+", "B-", "AB+", "AB-", "O+", "O-"];

const SHIRTS: &[&str] = &["XS", "S", "M", "L", "XL"];

const ONBOARDING: &[&str] =
    &["Completed", "In Progress", "Not Started", "Scheduled"];

const CLEARANCES: &[&str] = &["None", "Confidential", "Secret", "Top Secret"];

const STREETS: &[&str] = &[
    "Oak St",
    "Main St",
    "Elm Ave",
    "Park Blvd",
    "Cedar Ln",
    "Maple Dr",
    "Pine Rd",
    "Walnut St",
    "Broadway",
    "1st Ave",
    "2nd Ave",
    "3rd Ave",
];

// ── extra column system ──────────────────────────────

/// How to format the column in the grid.
#[derive(Clone, Copy)]
pub enum FormatHint {
    Text,
    Integer,
    Currency,
    Percent,
    Boolean,
    ImageText,
}

/// Generation strategy (private).
#[derive(Clone, Copy)]
enum Gen {
    /// Pick from a static array.
    Pick(&'static [&'static str]),
    /// Random integer in [lo, hi] inclusive.
    Int(i64, i64),
    /// Boolean with given percent-true.
    Bool(u32),
    /// US phone number.
    Phone,
    /// Random date YYYY-MM-DD.
    Date(u16, u16),
    /// @first.last handle.
    Handle,
    /// first-last-NNNNN profile ID.
    Profile,
    /// Street address.
    Address,
    /// Country: picks from COUNTRIES tuples, returns
    /// "CC Name" for CountryFlag format.
    Country,
    /// Gender: picks from GENDERS, returns
    /// "{icon_uri} {label}" for ImageText format.
    Gender,
}

/// Metadata for one extra column.
pub struct ExtraColDef {
    pub key: &'static str,
    pub label: &'static str,
    pub width: f64,
    pub format_hint: FormatHint,
    gen: Gen,
}

/// Shorthand constructor for table readability.
const fn ec(
    key: &'static str,
    label: &'static str,
    width: f64,
    format_hint: FormatHint,
    gen: Gen,
) -> ExtraColDef {
    ExtraColDef {
        key,
        label,
        width,
        format_hint,
        gen,
    }
}

// Aliases for table compactness.
const T: FormatHint = FormatHint::Text;
const I: FormatHint = FormatHint::Integer;
const C: FormatHint = FormatHint::Currency;
const P: FormatHint = FormatHint::Percent;
const B: FormatHint = FormatHint::Boolean;
const IT: FormatHint = FormatHint::ImageText;

/// 92 extra columns (indices 8..100).
pub static EXTRA_COLUMNS: &[ExtraColDef] = &[
    // ── Personal ────────────────────────────────
    ec("phone", "Phone", 150.0, T, Gen::Phone),
    ec("birth_date", "Birth Date", 110.0, T, Gen::Date(1960, 2000)),
    ec("hire_date", "Hire Date", 110.0, T, Gen::Date(2015, 2025)),
    ec("gender", "Gender", 130.0, IT, Gen::Gender),
    ec("address", "Address", 200.0, T, Gen::Address),
    ec("city", "City", 130.0, T, Gen::Pick(CITIES)),
    ec("state", "State", 60.0, T, Gen::Pick(STATES_US)),
    ec("country", "Country", 160.0, IT, Gen::Country),
    // ── Work ────────────────────────────────────
    ec("manager", "Manager", 180.0, T, Gen::Profile),
    ec("team", "Team", 100.0, T, Gen::Pick(TEAMS)),
    ec("office", "Office", 120.0, T, Gen::Pick(OFFICES)),
    ec("floor", "Floor", 60.0, I, Gen::Int(1, 30)),
    ec("badge_id", "Badge ID", 90.0, I, Gen::Int(10_000, 99_999)),
    ec("emp_type", "Type", 100.0, T, Gen::Pick(EMP_TYPES)),
    ec(
        "seniority",
        "Seniority",
        100.0,
        T,
        Gen::Pick(SENIORITY_LEVELS),
    ),
    ec("experience", "Exp. (yrs)", 80.0, I, Gen::Int(0, 35)),
    // ── Contact ─────────────────────────────────
    ec("mobile", "Mobile", 150.0, T, Gen::Phone),
    ec("extension", "Ext.", 70.0, I, Gen::Int(1000, 9999)),
    ec("slack", "Slack", 150.0, T, Gen::Handle),
    ec("linkedin", "LinkedIn", 180.0, T, Gen::Profile),
    // ── Performance ─────────────────────────────
    ec("rating", "Rating", 70.0, I, Gen::Int(1, 5)),
    ec(
        "last_review",
        "Last Review",
        110.0,
        T,
        Gen::Date(2024, 2026),
    ),
    ec("goals_met", "Goals Met", 80.0, P, Gen::Int(40, 100)),
    ec("projects_done", "Projects", 80.0, I, Gen::Int(0, 25)),
    ec("peer_score", "Peer Score", 90.0, I, Gen::Int(60, 100)),
    ec("promo_eligible", "Promo", 70.0, B, Gen::Bool(30)),
    // ── Financial ───────────────────────────────
    ec("bonus", "Bonus", 100.0, C, Gen::Int(0, 50_000)),
    ec("stock_opts", "Stock Opts", 100.0, I, Gen::Int(0, 10_000)),
    ec("benefits", "Benefits", 100.0, T, Gen::Pick(BENEFITS)),
    ec("tax_bracket", "Tax %", 70.0, P, Gen::Int(15, 37)),
    ec("commission", "Commission", 100.0, C, Gen::Int(0, 40_000)),
    ec(
        "total_comp",
        "Total Comp",
        120.0,
        C,
        Gen::Int(50_000, 400_000),
    ),
    // ── Skills ──────────────────────────────────
    ec("skill1", "Primary Skill", 130.0, T, Gen::Pick(SKILLS)),
    ec("skill2", "Secondary Skill", 130.0, T, Gen::Pick(SKILLS)),
    ec("cert", "Certification", 170.0, T, Gen::Pick(CERTS)),
    ec("language", "Language", 100.0, T, Gen::Pick(SPOKEN_LANGS)),
    ec("education", "Education", 110.0, T, Gen::Pick(EDU_LEVELS)),
    ec(
        "university",
        "University",
        150.0,
        T,
        Gen::Pick(UNIVERSITIES),
    ),
    // ── Time ────────────────────────────────────
    ec("hours_week", "Hrs/Wk", 70.0, I, Gen::Int(20, 60)),
    ec("vacation_days", "Vacation", 70.0, I, Gen::Int(0, 30)),
    ec("sick_days", "Sick Days", 70.0, I, Gen::Int(0, 12)),
    ec("overtime", "Overtime", 70.0, I, Gen::Int(0, 20)),
    ec("remote_days", "Remote/Wk", 70.0, I, Gen::Int(0, 5)),
    ec("timezone", "Timezone", 90.0, T, Gen::Pick(TIMEZONES)),
    // ── IT ──────────────────────────────────────
    ec("laptop", "Laptop", 140.0, T, Gen::Pick(LAPTOPS)),
    ec("os", "OS", 90.0, T, Gen::Pick(OSES)),
    ec("vpn_on", "VPN", 60.0, B, Gen::Bool(70)),
    ec("last_login", "Last Login", 110.0, T, Gen::Date(2026, 2026)),
    ec("disk_gb", "Disk (GB)", 80.0, I, Gen::Int(128, 2048)),
    ec("tickets_open", "Open Tix", 70.0, I, Gen::Int(0, 15)),
    // ── Metrics ─────────────────────────────────
    ec("cust_sat", "CSAT", 60.0, I, Gen::Int(1, 10)),
    ec("deals_closed", "Deals", 70.0, I, Gen::Int(0, 50)),
    ec("tix_resolved", "Resolved", 70.0, I, Gen::Int(0, 200)),
    ec("nps", "NPS", 60.0, I, Gen::Int(-100, 100)),
    ec("resp_time_ms", "Resp (ms)", 80.0, I, Gen::Int(50, 5000)),
    ec("uptime_pct", "Uptime %", 70.0, P, Gen::Int(95, 100)),
    ec("velocity", "Velocity", 70.0, I, Gen::Int(5, 40)),
    ec("bugs_fixed", "Bugs Fixed", 80.0, I, Gen::Int(0, 100)),
    // ── Background ──────────────────────────────
    ec(
        "prev_company",
        "Prev. Company",
        140.0,
        T,
        Gen::Pick(PREV_COMPANIES),
    ),
    ec(
        "referral",
        "Referral",
        130.0,
        T,
        Gen::Pick(REFERRAL_SOURCES),
    ),
    ec(
        "start_salary",
        "Start Salary",
        110.0,
        C,
        Gen::Int(35_000, 180_000),
    ),
    ec("relocations", "Relocations", 70.0, I, Gen::Int(0, 5)),
    ec("awards", "Awards", 70.0, I, Gen::Int(0, 8)),
    ec("publications", "Pubs", 60.0, I, Gen::Int(0, 20)),
    ec("patents", "Patents", 60.0, I, Gen::Int(0, 5)),
    ec("mentees", "Mentees", 70.0, I, Gen::Int(0, 10)),
    // ── Sales / Revenue ─────────────────────────
    ec("q_revenue", "Q Revenue", 110.0, C, Gen::Int(0, 500_000)),
    ec("pipeline", "Pipeline", 110.0, C, Gen::Int(0, 1_000_000)),
    ec("win_rate", "Win Rate", 70.0, P, Gen::Int(10, 80)),
    ec("avg_deal", "Avg Deal", 100.0, C, Gen::Int(5_000, 200_000)),
    ec("churn_pct", "Churn %", 70.0, P, Gen::Int(1, 25)),
    ec("ltv", "LTV", 100.0, C, Gen::Int(10_000, 500_000)),
    ec("arr", "ARR", 100.0, C, Gen::Int(50_000, 2_000_000)),
    ec("mrr", "MRR", 100.0, C, Gen::Int(4_000, 170_000)),
    // ── Misc / Preferences ──────────────────────
    ec("fav_lang", "Fav Language", 120.0, T, Gen::Pick(PROG_LANGS)),
    ec("ide", "IDE", 100.0, T, Gen::Pick(IDES)),
    ec("monitors", "Monitors", 70.0, I, Gen::Int(1, 4)),
    ec("standing", "Standing Desk", 70.0, B, Gen::Bool(45)),
    ec("coffee", "Coffee/Day", 70.0, I, Gen::Int(0, 8)),
    ec("meeting_hrs", "Mtg Hrs/Wk", 80.0, I, Gen::Int(2, 30)),
    ec("code_reviews", "Reviews/Wk", 80.0, I, Gen::Int(0, 20)),
    ec("deploys", "Deploys/Wk", 80.0, I, Gen::Int(0, 15)),
    // ── Extra ───────────────────────────────────
    ec("blood_type", "Blood Type", 80.0, T, Gen::Pick(BLOOD_TYPES)),
    ec("shirt_size", "Shirt", 60.0, T, Gen::Pick(SHIRTS)),
    ec("onboarding", "Onboarding", 110.0, T, Gen::Pick(ONBOARDING)),
    ec("clearance", "Clearance", 100.0, T, Gen::Pick(CLEARANCES)),
];

// ── generation ───────────────────────────────────────

fn generate_extra(row: u64, col_key: &str) -> Option<String> {
    let (idx, col) = EXTRA_COLUMNS
        .iter()
        .enumerate()
        .find(|(_, c)| c.key == col_key)?;
    let salt = 100 + idx as u64;
    Some(match col.gen {
        Gen::Pick(arr) => pick(arr, row, salt).to_owned(),
        Gen::Int(lo, hi) => {
            let range = (hi - lo + 1) as u64;
            let val = lo + (hash_field(row, salt) % range) as i64;
            val.to_string()
        }
        Gen::Bool(pct) => {
            let v = (hash_field(row, salt) % 100) < pct as u64;
            v.to_string()
        }
        Gen::Phone => {
            let h = hash_field(row, salt);
            let area = 200 + (h % 800) as u16;
            let mid = 100 + ((h >> 16) % 900) as u16;
            let end = 1000 + ((h >> 32) % 9000) as u16;
            format!("+1 ({area:03}) {mid:03}-{end:04}")
        }
        Gen::Date(y_min, y_max) => {
            let h = hash_field(row, salt);
            let range = (y_max - y_min + 1) as u64;
            let y = y_min as u64 + h % range;
            let m = 1 + (h >> 20) % 12;
            let d = 1 + (h >> 28) % 28;
            format!("{y:04}-{m:02}-{d:02}")
        }
        Gen::Handle => {
            let f = pick(FIRST_NAMES, row, salt);
            let l = pick(LAST_NAMES, row, salt + 50);
            format!("@{}.{}", f.to_lowercase(), l.to_lowercase())
        }
        Gen::Profile => {
            let f = pick(FIRST_NAMES, row, salt);
            let l = pick(LAST_NAMES, row, salt + 50);
            let n = hash_field(row, salt + 99) % 100_000;
            format!("{}-{}-{n:05}", f.to_lowercase(), l.to_lowercase())
        }
        Gen::Address => {
            let h = hash_field(row, salt);
            let num = 1 + (h % 9999) as u16;
            let street = pick(STREETS, row, salt + 50);
            format!("{num} {street}")
        }
        Gen::Country => {
            let idx = hash_field(row, salt) as usize % COUNTRIES.len();
            let (code, name) = COUNTRIES[idx];
            let uri = rs_grid_icons::flag_data_uri(code).unwrap_or("");
            format!("{uri} {name}")
        }
        Gen::Gender => {
            let label = pick(GENDERS, row, salt);
            let key = label.to_uppercase().replace(' ', "-");
            let uri = rs_grid_icons::gender_icon_uri(&key).unwrap_or("");
            format!("{uri} {label}")
        }
    })
}

// ── dynamic columns (beyond the 92 hand-crafted ones) ─

/// Templates cycled through for generated columns.
const DYN_TEMPLATES: &[(&str, FormatHint, Gen)] = &[
    ("Score", I, Gen::Int(0, 100)),
    ("Amount", C, Gen::Int(100, 99_999)),
    ("Rate", P, Gen::Int(1, 100)),
    ("Enabled", B, Gen::Bool(50)),
    ("Count", I, Gen::Int(0, 500)),
    ("Value", C, Gen::Int(1_000, 999_999)),
    ("Ratio", P, Gen::Int(5, 95)),
    ("Flag", B, Gen::Bool(70)),
    ("Qty", I, Gen::Int(1, 1_000)),
    ("Total", C, Gen::Int(0, 50_000)),
];

/// Number of hand-crafted extra columns.
pub const EXTRA_COUNT: usize = 92;

/// Return column metadata for a dynamic column index
/// (0-based, relative to the first dynamic column).
pub fn dynamic_col_def(dyn_idx: usize) -> (String, String, f64, FormatHint) {
    let tpl = &DYN_TEMPLATES[dyn_idx % DYN_TEMPLATES.len()];
    let n = dyn_idx + 1;
    let key = format!("dyn_{n}");
    let label = format!("{} {n}", tpl.0);
    (key, label, 90.0, tpl.1)
}

/// Generate a value for a dynamic column.
fn generate_dynamic(row: u64, dyn_idx: usize) -> String {
    let tpl = &DYN_TEMPLATES[dyn_idx % DYN_TEMPLATES.len()];
    let salt = 1000 + dyn_idx as u64;
    match tpl.2 {
        Gen::Int(lo, hi) => {
            let range = (hi - lo + 1) as u64;
            let val = lo + (hash_field(row, salt) % range) as i64;
            val.to_string()
        }
        Gen::Bool(pct) => {
            let v = (hash_field(row, salt) % 100) < pct as u64;
            v.to_string()
        }
        _ => String::new(),
    }
}

// ── public API ────────────────────────────────────────

/// Return a fake cell value for the given row and
/// column key. Deterministic: same (row, col_key)
/// always returns the same value.
pub fn fake_cell(row: u64, col_key: &str) -> Option<String> {
    match col_key {
        "name" => {
            let first = pick(FIRST_NAMES, row, 1);
            let last = pick(LAST_NAMES, row, 2);
            Some(format!("{first} {last}"))
        }
        "email" => {
            let first = pick(FIRST_NAMES, row, 1);
            let last = pick(LAST_NAMES, row, 2);
            Some(format!(
                "{}.{}@example.com",
                first.to_lowercase(),
                last.to_lowercase(),
            ))
        }
        "role" => {
            let idx = hash_field(row, 3) as usize % ROLES.len();
            Some(ROLES[idx].0.to_owned())
        }
        "dept" => Some(pick(DEPARTMENTS, row, 4).to_owned()),
        "salary" => {
            let idx = hash_field(row, 3) as usize % ROLES.len();
            let (_, lo, hi) = ROLES[idx];
            let range = hi - lo;
            let offset = hash_field(row, 5) as u32 % (range + 1);
            let salary = ((lo + offset) / 1_000) * 1_000;
            Some(salary.to_string())
        }
        "avatar" => {
            let first = pick(FIRST_NAMES, row, 1);
            let last = pick(LAST_NAMES, row, 2);
            Some(format!("{first}+{last}"))
        }
        "active" => {
            let h = hash_field(row, 6);
            let active = (h % 100) >= 15;
            Some(active.to_string())
        }
        _ => {
            if let Some(s) = col_key.strip_prefix("dyn_") {
                if let Ok(n) = s.parse::<usize>() {
                    return Some(generate_dynamic(row, n.saturating_sub(1)));
                }
            }
            generate_extra(row, col_key)
        }
    }
}
