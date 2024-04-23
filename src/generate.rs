use kalosm::language::*;

pub async fn generate() {
    // What datatype should be inside the {{}}s in this code?
    // <p>{{replace with a brief description of Kalosm, its purpose, and what sets it apart from other open-source frameworks for language, audio, and image models in Rust.}}</p>
    // Respond with String, signed int, unsigned int, or float

    let llm = Llama::new_chat().await.unwrap();

    let task = Task::builder(r#"You generate completely static snippets of HTML. You will only use tailwindcss for styling.

Terminology:
- Component: A self contained static piece of HTML that can be reused in multiple places
- Children: Any HTML nodes that are nested inside a component

You always follow this response format:
1) What should the UI look like?
2) What are the individual components that make up the UI? Name each component with an upper camel case identifier.
3) What does the HTML for the UI look like? Insert `<ComponentName/>` in the HTML where you want the component to be inserted.
4) What is the HTML for each component? Components may render child elements with the special `<Children/>` tag.

For any information you don't know in the HTML. Use `{lower_camel_case_identifier}` in the HTML instead of the information. The information must be a string or number only.
For example, if you don't know how many downloads a library has, you might put <p>{download_count} downloads</p> in the HTML."#)
.with_example(r#"A github star counter"#, 
r#"
1. What should the UI look like?
The UI should be a simple login form with a header, a form with input fields for username and password, and a submit button.

2. What are the individual components that make up the UI?
The components are:
- Header
- Form
- FormHeader
- FormBody
- SubmitButton

3. What does the HTML for the UI look like?
```html
<div class="h-screen flex justify-center items-center">
  <div class="bg-white p-6 shadow-md w-full max-w-md">
    <Header/>
    <Form>
      <FormHeader>Login</FormHeader>
      <FormBody>
        <label class="block text-sm font-medium text-gray-700">
          Username
        </label>
        <input class="block w-full mt-1 text-sm text-gray-700" type="text" placeholder="Username" />
        <label class="block text-sm font-medium text-gray-700">
          Password
        </label>
        <input class="block w-full mt-1 text-sm text-gray-700" type="password" placeholder="Password" />
      </FormBody>
      <SubmitButton>Login</SubmitButton>
    </Form>
  </div>
</div>
```
4. What is the HTML for each component?
Header: 
```html
<div class="bg-gray-200 p-4 text-lg font-bold">Login Form</div>
```
Form:
```html
<div class="bg-white p-6 shadow-md w-full max-w-md">
  <FormHeader/>
  <FormBody/>
  <SubmitButton/>
</div>
```
FormHeader:
```html
<h2 class="text-lg font-bold">Login</h2>
```
FormBody:
```html
<div class="flex flex-col">
  <label class="block text-sm font-medium text-gray-700">
    {username_label}
  </label>
  <input class="block w-full mt-1 text-sm text-gray-700" type="text" placeholder="Username" />
  <label class="block text-sm font-medium text-gray-700">
    {password_label}
  </label>
  <input class="block w-full mt-1 text-sm text-gray-700" type="password" placeholder="Password" />
</div>
```
SubmitButton:
```html
<button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
  {submit_button_text}
</button>
```
"#)
        .build();

    loop {
        let input = prompt_input("What do you want to make? ").unwrap();
        let start_timestamp = std::time::Instant::now();
        task.run(input, &llm).to_std_out().await.unwrap();
        println!("\nTook: {:?}", start_timestamp.elapsed());
    }
}
