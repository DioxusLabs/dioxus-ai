fn main() {
    // let expected_html = Prompt {
    //     previous: r#"<div class="Day-styles__root planner-day"><h2 dir="ltr" class="css-tz46fa-view-heading"><div class="Day-styles__secondary">Tuesday, April 23</div></h2><div><div class="NotificationBadge-styles__activityIndicator"></div><div class="PlannerItem-styles__completed"><div class="css-3orjyc-checkbox"><input id="Checkbox_1" type="checkbox" class="css-22ol9d-checkbox__input" value=""><label for="Checkbox_1" class="css-1h7tu9i-checkbox__control"><span class="css-1v5k22r-checkboxFacade"><span aria-hidden="true" class="css-uxghep-checkboxFacade__facade"></span><span class="css-1eh1zrl-checkboxFacade__label"><span class="css-1sr5vj2-screenReaderContent">Assignment Homework 4 is not marked as done.</span></span></span></label></div></div><div class="PlannerItem-styles__icon" aria-hidden="true" style="color: rgb(79, 128, 69);"><svg name="IconAssignment" viewBox="0 0 1920 1920" rotate="0" width="1em" height="1em" aria-hidden="true" role="presentation" focusable="false" class="css-1xnn9jb-inlineSVG-svgIcon" style="width: 1em; height: 1em;"><g role="presentation"><path d="M1468.214 0v564.698h-112.94V112.94H112.94v1694.092h1242.334v-225.879h112.94v338.819H0V0h1468.214Zm129.428 581.311c22.137-22.136 57.825-22.136 79.962 0l225.879 225.879c22.023 22.023 22.023 57.712 0 79.848l-677.638 677.637c-10.616 10.504-24.96 16.49-39.98 16.49h-225.88c-31.17 0-56.469-25.299-56.469-56.47v-225.88c0-15.02 5.986-29.364 16.49-39.867Zm-155.291 314.988-425.895 425.895v146.031h146.03l425.895-425.895-146.03-146.03Zm-764.714 346.047v112.94H338.82v-112.94h338.818Zm225.88-225.88v112.94H338.818v-112.94h564.697Zm734.106-315.44-115.424 115.425 146.03 146.03 115.425-115.423-146.031-146.031ZM1129.395 338.83v451.758H338.82V338.83h790.576Zm-112.94 112.94H451.759v225.878h564.698V451.77Z" fill-rule="evenodd"></path></g></svg></div><div class="PlannerItem-styles__layout"><div class="PlannerItem-styles__innerLayout"><div class="PlannerItem-styles__details PlannerItem-styles__details_no_badges"><div class="PlannerItem-styles__type"><span color="secondary" wrap="normal" letter-spacing="normal" class="css-qbjizl-text">4242-40674:EECS 388 Embedded Systems LEC Assignment</span></div><a dir="ltr" href="/courses/109629/assignments/799798" class="css-1xynqde-view-link"><span class="css-1sr5vj2-screenReaderContent">Assignment Homework 4, due Tuesday, April 23, 2024 11:59 PM.</span><span aria-hidden="true">Homework 4</span></a></div><div class="PlannerItem-styles__secondary PlannerItem-styles__secondary_no_badges"><div class="PlannerItem-styles__badges"></div><div class="PlannerItem-styles__metrics"><div class="PlannerItem-styles__score"><span wrap="normal" letter-spacing="normal" class="css-mum2ig-text">2</span><span wrap="normal" letter-spacing="normal" class="css-1uakmj8-text">&nbsp;pts</span></div><div class="PlannerItem-styles__due"><span wrap="normal" letter-spacing="normal" class="css-1uakmj8-text"><span aria-hidden="true">Due: 11:59 PM</span></span></div></div></div></div></div></div></li></ol></div></div></div>"#.to_string(),
    //     action: "clicked on the checkbox".to_string(),
    //     new: r#"<div class="Day-styles__root planner-day"><h2 dir="ltr" class="css-tz46fa-view-heading"><div class="Day-styles__secondary">Tuesday, April 23</div></h2><div><div class="NotificationBadge-styles__activityIndicator"></div><div class="PlannerItem-styles__completed"><div class="css-3orjyc-checkbox"><input id="Checkbox_1" type="checkbox" class="css-22ol9d-checkbox__input" value=""><label for="Checkbox_1" class="css-1h7tu9i-checkbox__control"><span class="css-1v5k22r-checkboxFacade"><span aria-hidden="true" class="css-4vmk4b-checkboxFacade__facade"><svg name="IconCheckMark" viewBox="0 0 1920 1920" rotate="0" width="1em" height="1em" aria-hidden="true" role="presentation" focusable="false" class="css-8x7cbx-inlineSVG-svgIcon" style="width: 1em; height: 1em;"><g role="presentation"><path d="M1743.858 267.012 710.747 1300.124 176.005 765.382 0 941.387l710.747 710.871 1209.24-1209.116z" fill-rule="evenodd"></path></g></svg></span><span class="css-1eh1zrl-checkboxFacade__label"><span class="css-1sr5vj2-screenReaderContent">Assignment Homework 4 is marked as done.</span></span></span></label></div></div><div class="PlannerItem-styles__icon" aria-hidden="true" style="color: rgb(79, 128, 69);"><svg name="IconAssignment" viewBox="0 0 1920 1920" rotate="0" width="1em" height="1em" aria-hidden="true" role="presentation" focusable="false" class="css-1xnn9jb-inlineSVG-svgIcon" style="width: 1em; height: 1em;"><g role="presentation"><path d="M1468.214 0v564.698h-112.94V112.94H112.94v1694.092h1242.334v-225.879h112.94v338.819H0V0h1468.214Zm129.428 581.311c22.137-22.136 57.825-22.136 79.962 0l225.879 225.879c22.023 22.023 22.023 57.712 0 79.848l-677.638 677.637c-10.616 10.504-24.96 16.49-39.98 16.49h-225.88c-31.17 0-56.469-25.299-56.469-56.47v-225.88c0-15.02 5.986-29.364 16.49-39.867Zm-155.291 314.988-425.895 425.895v146.031h146.03l425.895-425.895-146.03-146.03Zm-764.714 346.047v112.94H338.82v-112.94h338.818Zm225.88-225.88v112.94H338.818v-112.94h564.697Zm734.106-315.44-115.424 115.425 146.03 146.03 115.425-115.423-146.031-146.031ZM1129.395 338.83v451.758H338.82V338.83h790.576Zm-112.94 112.94H451.759v225.878h564.698V451.77Z" fill-rule="evenodd"></path></g></svg></div><div class="PlannerItem-styles__layout"><div class="PlannerItem-styles__innerLayout"><div class="PlannerItem-styles__details PlannerItem-styles__details_no_badges"><div class="PlannerItem-styles__type"><span color="secondary" wrap="normal" letter-spacing="normal" class="css-qbjizl-text">4242-40674:EECS 388 Embedded Systems LEC Assignment</span></div><a dir="ltr" href="/courses/109629/assignments/799798" class="css-1xynqde-view-link"><span class="css-1sr5vj2-screenReaderContent">Assignment Homework 4, due Tuesday, April 23, 2024 11:59 PM.</span><span aria-hidden="true">Homework 4</span></a></div><div class="PlannerItem-styles__secondary PlannerItem-styles__secondary_no_badges"><div class="PlannerItem-styles__badges"></div><div class="PlannerItem-styles__metrics"><div class="PlannerItem-styles__score"><span wrap="normal" letter-spacing="normal" class="css-mum2ig-text">2</span><span wrap="normal" letter-spacing="normal" class="css-1uakmj8-text">&nbsp;pts</span></div><div class="PlannerItem-styles__due"><span wrap="normal" letter-spacing="normal" class="css-1uakmj8-text"><span aria-hidden="true">Due: 11:59 PM</span></span></div></div></div></div></div></div></li></ol></div></div></div>"#.to_string(),
    // };

    // task.run(&expected_html.to_string(), &llm)
    //     .to_std_out()
    //     .await
    //     .unwrap();

    // let unexpected = Prompt {
    //     previous: r#"[Python](button)
    // [JavaScript](button)
    // [cURL](button)

    // import replicate

    // output = replicate.run(
    // "ai-forever/kandinsky-2.2:ea1addaab376f4dc227f5368bbd8eff901820fd1cc14ed8cad63b29249e9d463",
    // input={
    // "prompt": "A moss covered astronaut with a black background"
    // }
    // )

    // print(output)"#
    //         .to_string(),
    //     action: "clicked on the JavaScript button".to_string(),
    //     new: r#"[Python](button)
    // [JavaScript](button)
    // [cURL](button)

    // curl -s -X POST
    // -H "Authorization: Token $REPLICATE_API_TOKEN"
    // -H "Content-Type: application/json"
    // -d $'{
    // "version": "39ed52f2a78e934b3ba6e2a89f5b1c712de7dfea535525255b1aa35c5565e08b",
    // "input": {
    // "prompt": "A moss covered astronaut with a black background"
    // }
    // }'
    // https://api.replicate.com/v1/predictions"#
    //         .to_string(),
    // };

    // task.run(&unexpected.to_string(), &llm)
    //     .to_std_out()
    //     .await
    //     .unwrap();

    // let expected = Prompt {
    //     previous: r#"[Python](button)
    // [JavaScript](button)
    // [cURL](button)

    // import replicate

    // output = replicate.run(
    // "ai-forever/kandinsky-2.2:ea1addaab376f4dc227f5368bbd8eff901820fd1cc14ed8cad63b29249e9d463",
    // input={
    // "prompt": "A moss covered astronaut with a black background"
    // }
    // )

    // print(output)"#
    //         .to_string(),
    //     action: "clicked on the cURL button".to_string(),
    //     new: r#"[Python](button)
    // [JavaScript](button)
    // [cURL](button)

    // curl -s -X POST
    // -H "Authorization: Token $REPLICATE_API_TOKEN"
    // -H "Content-Type: application/json"
    // -d $'{
    // "version": "39ed52f2a78e934b3ba6e2a89f5b1c712de7dfea535525255b1aa35c5565e08b",
    // "input": {
    // "prompt": "A moss covered astronaut with a black background"
    // }
    // }'
    // https://api.replicate.com/v1/predictions"#
    //         .to_string(),
    // };

    // task.run(&expected.to_string(), &llm)
    //     .to_std_out()
    //     .await
    //     .unwrap();
}
