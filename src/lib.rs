use leptos::{error::Result, ev::SubmitEvent, html::Input, *};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

impl Todo {
    fn new() -> Self {
        Todo {
            id: 0,
            text: "".to_string(),
            completed: false,
        }
    }
}

#[derive(Error, Clone, Debug)]
pub enum TodoError {
    #[error("Please request more than zero todos.")]
    NonZeroTodos,
}

type TodoCount = usize;

async fn fetch_todos(count: TodoCount) -> Result<Vec<Todo>> {
    if count > 0 {
        // make the request
        let res = reqwasm::http::Request::get("http://localhost:3000/todos")
            .send()
            .await?
            // convert it to JSON
            .json::<Vec<Todo>>()
            .await?
            .into_iter()
            .take(count)
            .collect::<Vec<_>>();
        // // extract the URL field for each cat
        // .into_iter()
        // .take(count)
        // .map(|cat| cat.url)
        // .collect::<Vec<_>>();
        Ok(res)
    } else {
        Err(TodoError::NonZeroTodos.into())
    }
}

async fn delete_todo(id: i32) -> Result<String> {
    if id != 0 {
        let body = Client::new()
            .delete(format!("http://localhost:3000/todos/{}", id))
            .send()
            .await?
            .text()
            .await?;
        println!("deleted todo = {body}");
        Ok(body)
    } else {
        println!("nothing to delete");
        Ok("no delete".to_string())
    }
}

async fn create_todo(text: String) -> Result<String> {
    if text != "" {
        let mut map = HashMap::new();
        map.insert("text", text);
        let body = Client::new()
            .post("http://localhost:3000/todos")
            .json(&map)
            .send()
            .await?
            .text()
            .await?;
        println!("created todo = {body}");
        Ok(body)
    } else {
        println!("nothing to create");
        Ok("no create".to_string())
    }
}

async fn update_todo_completed(todo: Todo) -> Result<String> {
    let mut map = HashMap::new();
    map.insert("completed", todo.completed);
    let body = Client::new()
        .put(format!("http://localhost:3000/todos/{}", todo.id))
        .json(&map)
        .send()
        .await?
        .text()
        .await?;
    println!("updated todo completed = {body}");
    Ok(body)
}

async fn update_todo_text(todo: Todo) -> Result<String> {
    let mut map = HashMap::new();
    map.insert("text", todo.text);
    let body = Client::new()
        .put(format!("http://localhost:3000/todos/{}", todo.id))
        .json(&map)
        .send()
        .await?
        .text()
        .await?;
    println!("updated todo text = {body}");
    Ok(body)
}

pub fn show_todos() -> impl IntoView {
    let (todo_count, set_todo_count) = create_signal::<TodoCount>(1);
    let (delete_id, set_delete_id) = create_signal::<i32>(0);
    let (new_todo_text, set_new_todo_text) = create_signal("".to_string());
    let (todo_completed, set_todo_completed) = create_signal::<Todo>(Todo::new());
    let (todo_text, set_todo_text) = create_signal::<Todo>(Todo::new());

    // we'll use a NodeRef to store a reference to the input element
    // this will be filled when the element is created
    let new_todo_text_input_element: NodeRef<Input> = create_node_ref();

    // fires when the form `submit` event happens
    // this will store the value of the <input> in our signal
    let on_submit_new_todo_text = move |ev: SubmitEvent| {
        // stop the page from reloading!
        ev.prevent_default();

        // here, we'll extract the value from the input
        let value = new_todo_text_input_element()
            // event handlers can only fire after the view
            // is mounted to the DOM, so the `NodeRef` will be `Some`
            .expect("<input> to exist")
            // `NodeRef` implements `Deref` for the DOM element type
            // this means we can call`HtmlInputElement::value()`
            // to get the current value of the input
            .value();
        set_new_todo_text(value);
    };

    // we'll use a NodeRef to store a reference to the input element
    // this will be filled when the element is created
    let todo_text_input_element: NodeRef<Input> = create_node_ref();

    // fires when the form `submit` event happens
    // this will store the value of the <input> in our signal
    let on_submit_update_todo_text = move |ev: SubmitEvent| {
        // stop the page from reloading!
        ev.prevent_default();

        // here, we'll extract the value from the input
        let value = todo_text_input_element()
            // event handlers can only fire after the view
            // is mounted to the DOM, so the `NodeRef` will be `Some`
            .expect("<input> to exist")
            // `NodeRef` implements `Deref` for the DOM element type
            // this means we can call`HtmlInputElement::value()`
            // to get the current value of the input
            .value();
        set_todo_text.update(|tu| {
            *tu = Todo {
                id: tu.id,
                text: value,
                completed: tu.completed,
            }
        });
    };

    // we use local_resource here because
    // 1) our error type isn't serializable/deserializable
    // 2) we're not doing server-side rendering in this example anyway
    //    (during SSR, create_resource will begin loading on the server and resolve on the client)
    let todos = create_local_resource(todo_count, fetch_todos);
    let deleted_todo = create_local_resource(delete_id, delete_todo);
    let new_todo = create_local_resource(new_todo_text, create_todo);
    let completed_todo = create_local_resource(todo_completed, update_todo_completed);
    let updated_todo = create_local_resource(todo_text, update_todo_text);

    let fallback = move |errors: RwSignal<Errors>| {
        let error_list = move || {
            errors.with(|errors| {
                errors
                    .iter()
                    .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                    .collect_view()
            })
        };

        view! {
            <div class="error">
                <h2>"Error"</h2>
                <ul>{error_list}</ul>
            </div>
        }
    };

    // the renderer can handle Option<_> and Result<_> states
    // by displaying nothing for None if the resource is still loading
    // and by using the ErrorBoundary fallback to catch Err(_)
    // so we'll just use `.and_then()` to map over the happy path
    let todos_view = move || {
        todos.and_then(|data| {
            data.iter()
                // .map(|todo| view! { <li> {todo.id} | {todo.text.clone()} | {todo.completed} </li> })
                .map(|todo| {
                    let t = todo.clone();
                    view! {
                         {todo.id}<br/>
                         // {todo.text.clone()}<br/>
                         {&todo.text}<br/>
                         {todo.completed}<br/>
                         // <form on:submit=on_submit_update_todo_text>
                         //    <input type="text"
                         //        // here, we use the `value` *attribute* to set only
                         //        // the initial value, letting the browser maintain
                         //        // the state after that
                         //        value=todo_text
                         //
                         //        // store a reference to this input in `input_element`
                         //        node_ref=todo_text_input_element
                         //    />
                         //    <input type="submit" value="Update"/>
                         // </form>
                         // <p>"Todo text is: " {todo_text.get().text}</p>
                        <label>
                            "Update text: "
                            <input
                                type="text"
                                prop:value=move || "".to_string()
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_todo_text(Todo {
                                        id: t.id,
                                        text: val,
                                        completed: t.completed,
                                    })
                                }
                            />
                        </label>
                         <button on:click=move |_| {
                                set_delete_id.update(|n| *n = t.id);
                                // todos.refetch();
                                // set_todo_count.update(|n| *n -= 1);
                                // set_todo_count(todo_count.get() - 1);
                            }>
                            "Delete"
                         </button>
                         <button on:click=move |_| {
                                set_todo_completed.update(|tc| {
                                        *tc = Todo {
                                            id: t.id,
                                            text: t.text.clone(),
                                            completed: true,
                                        }
                                    });
                            }>
                            "Complete"
                         </button>
                         <br/>
                         <br/>
                    }
                })
                .collect_view()
        })
    };

    view! {
        <div>
            <label>
                "How many todos would you like? "
                <input
                    type="number"
                    prop:value=move || todo_count.get().to_string()
                    on:input=move |ev| {
                        let val = event_target_value(&ev).parse::<TodoCount>().unwrap_or(0);
                        set_todo_count(val);
                    }
                />
            </label>
            <br/>
            <br/>
            <p>
                 <button on:click=move |_| {
                        todos.refetch();
                        // set_todo_count.update(|n| *n -= 1);
                        // set_todo_count(todo_count.get() - 1);
                    }>
                    "Refresh"
                 </button>
            </p>
            <br/>
            <br/>
            <ErrorBoundary fallback>
                <Transition fallback=move || {
                    view! { <div>"Loading (Suspense Fallback)..."</div> }
                }>
                <div>
                    {todos_view}
                </div>
                </Transition>
            </ErrorBoundary>
            <br/>
            <br/>
            New Todo:
            <form on:submit=on_submit_new_todo_text>
                <input type="text"
                    // here, we use the `value` *attribute* to set only
                    // the initial value, letting the browser maintain
                    // the state after that
                    value=new_todo_text

                    // store a reference to this input in `input_element`
                    node_ref=new_todo_text_input_element
                />
                <input type="submit" value="Submit"/>
            </form>
            <p>"Todo text is: " {new_todo_text}</p>
        </div>
    }
}
