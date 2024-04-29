use kalosm::language::*;

// TODO: This version seems to generate JSX instead of HTML. This could be fixed with constraints
const CHILDREN_AND_COMPONENTS: &str = r#"You generate snippets of JSX. You will only use tailwindcss for styling. You may include comments to explain the HTML with the `<!-- comment -->` syntax.

You always follow this response format:
1) What should the UI look like?
2) What are the individual components that make up the UI? Name each component with an upper camel case identifier and specify if the component is standalone or takes children.
3) What does the HTML for the UI look like? Insert `<ComponentName/>` in the HTML where you want the component to be inserted. This **must** be valid HTML. May not contain `{Children}` placeholder.
4) What is the HTML for each component? Components may render child elements with the special `{children}` placeholder.

For any information you don't know in the HTML. Use `{lower_camel_case_identifier}` in the HTML instead of the information. The information must be a string or number only.
For example, if you don't know how many downloads a library has, you might put <p>{download_count} downloads</p> in the HTML."#;

const TAILWIND_UI: &str = r#"You generate snippets of HTML with a focus on clear layout and clean design. You avoid images and graphics. You will only use tailwindcss for styling. For any information you don't know in the HTML. Use `{lower_camel_case_identifier}` in the HTML instead of the information. The information must be a string or number only.
You use the jsx syntax for templating. For example, if you don't know how many downloads a library has, you might put <p>{{download_count}} downloads</p> in the HTML.

You always follow this response format:
1) What should the UI look like?
2) What is the HTML for that UI?"#;

pub async fn generate() {
    let llm = Llama::new_chat().await.unwrap();

    let task = Task::builder(CHILDREN_AND_COMPONENTS)
.with_example(r#"A github star counter"#, 
r#"**What should the UI look like?**

A simple, clean, and minimalistic design that displays the number of stars a GitHub repository has received. The UI should have a subtle background color, with a clear and readable font to display the star count.

**What is the HTML for that UI?**

```
<div class="bg-gray-100 p-4 rounded-md">
  <p class="text-lg font-bold">{star_count} stars</p>
</div>
```

In this HTML, `{star_count}` represents the actual number of stars the repository has received."#)
        .build();

    loop {
        let input = prompt_input("What do you want to make? ").unwrap();
        let start_timestamp = std::time::Instant::now();
        task.run(input, &llm).to_std_out().await.unwrap();
        println!("\nTook: {:?}", start_timestamp.elapsed());
    }
}
