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

// ── static data ───────────────────────────────────────

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
    // Executive
    ("CEO", 250_000, 350_000),
    ("CTO", 220_000, 320_000),
    ("CFO", 210_000, 300_000),
    ("VP of Engineering", 180_000, 260_000),
    ("VP of Sales", 175_000, 250_000),
    // Senior
    ("Senior Software Engineer", 140_000, 200_000),
    ("Engineering Manager", 150_000, 210_000),
    ("Director of Marketing", 130_000, 190_000),
    ("Principal Engineer", 160_000, 220_000),
    ("Senior Product Manager", 135_000, 195_000),
    // Mid
    ("Software Engineer", 90_000, 140_000),
    ("Product Manager", 95_000, 145_000),
    ("Data Analyst", 80_000, 120_000),
    ("UX Designer", 85_000, 130_000),
    ("Marketing Manager", 80_000, 125_000),
    ("Account Executive", 75_000, 130_000),
    // Junior
    ("Junior Developer", 55_000, 80_000),
    ("Sales Associate", 45_000, 70_000),
    ("Marketing Coordinator", 48_000, 72_000),
    ("Support Specialist", 42_000, 65_000),
    ("QA Analyst", 50_000, 78_000),
];

// ── public API ────────────────────────────────────────

/// Return a fake cell value for the given row and
/// column key. Deterministic: same (row, col_key)
/// always returns the same value.
pub fn fake_cell(row: u64, col_key: &str) -> Option<String> {
    match col_key {
        "id" => Some(row.to_string()),
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
        _ => None,
    }
}
