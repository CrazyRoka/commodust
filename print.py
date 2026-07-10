from pathlib import Path

def print_md_files_and_content(directory="."):
    path = Path(directory)
    
    # Find all .md files recursively
    md_files = list(path.rglob("*.md"))
    
    if not md_files:
        print("No markdown files found.")
        return

    for file_path in md_files:
        print("\n" + "="*60)
        print(f"FILE: {file_path}")
        print("="*60 + "\n")
        
        try:
            # Open and read the file content
            # encoding='utf-8' prevents crashes on emojis or special characters
            content = file_path.read_text(encoding="utf-8")
            print(content)
        except Exception as e:
            print(f"[Error reading file]: {e}")
            
        print("\n" + "-"*60)

if __name__ == "__main__":
    print_md_files_and_content()