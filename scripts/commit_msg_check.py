import argparse
import re
import sys


def check_no_ai_attribution(msg: str) -> str | None:
    if re.search(r'co-authored-by.*(claude|anthropic)', msg, re.IGNORECASE):
        return "Commit message must not contain AI co-author attribution."
    return None


def check_subject_ends_with_period(msg: str) -> str | None:
    subject = msg.splitlines()[0]
    if not subject.endswith('.'):
        return "Subject line must end with a period."
    return None


def check_blank_line_after_subject(msg: str) -> str | None:
    lines = msg.splitlines()
    if len(lines) > 1 and lines[1].strip() != '':
        return "Second line of commit message must be blank."
    return None


def validate(msg_file: str) -> list[str]:
    msg = open(msg_file).read()
    checks = [check_no_ai_attribution, check_subject_ends_with_period, check_blank_line_after_subject]
    return [err for check in checks if (err := check(msg))]


def main() -> None:
    parser = argparse.ArgumentParser(description="Validate a git commit message.")
    parser.add_argument("msg_file", help="Path to the commit message file.")
    args = parser.parse_args()

    errors = validate(args.msg_file)
    for err in errors:
        print(f"✗ {err}")
    sys.exit(1 if errors else 0)


if __name__ == "__main__":
    main()
