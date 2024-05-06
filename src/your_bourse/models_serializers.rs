use chrono::Utc;
use my_nosql_contracts::YbPriceFeedSettings;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use rust_fix::FixMessageWriter;

const OUR_FIX_VERSION: &'static str = "FIX.4.4";

pub fn serialize_ping(settings: &YbPriceFeedSettings, count: u64) -> FixMessageWriter {
    let date = Utc::now();
    let date_string = date.format("%Y%m%d-%H:%M:%S.%3f").to_string();

    let mut fix_builder = FixMessageWriter::new(OUR_FIX_VERSION, "0");
    fix_builder.with_value("49", &settings.sender_company_id);
    fix_builder.with_value("52", date_string.as_str());
    fix_builder.with_value("56", &settings.target_company_id);
    fix_builder.with_value("34", count.to_string().as_str());

    return fix_builder;
}

pub fn serialize_logon(settings: &YbPriceFeedSettings, count: u64) -> FixMessageWriter {
    let date = Utc::now();
    let date_string = date.format("%Y%m%d-%H:%M:%S.%3f").to_string();

    let mut fix_builder = FixMessageWriter::new(OUR_FIX_VERSION, "A");
    fix_builder.with_value("49", &settings.sender_company_id);
    fix_builder.with_value("56", &settings.target_company_id);
    fix_builder.with_value("52", date_string.as_str());
    fix_builder.with_value("34", count.to_string().as_str());
    fix_builder.with_value("108", "30");
    fix_builder.with_value("141", "Y");
    fix_builder.with_value("554", &settings.pass);
    fix_builder.with_value("98", "0");

    println!("Logon message: {}", fix_builder.to_string());
    return fix_builder;
}

pub fn serialize_instrument_subscribe(
    settings: &YbPriceFeedSettings,
    count: u64,
    instrument: &str,
) -> FixMessageWriter {
    let now = DateTimeAsMicroseconds::now();
    let date_string = crate::date_utils::to_fix_date_string(now);

    let mut fix_builder = FixMessageWriter::new(OUR_FIX_VERSION, "V");
    let uuid = now.unix_microseconds;

    fix_builder.with_value("49", &settings.sender_company_id);
    fix_builder.with_value("52", date_string.as_str());
    fix_builder.with_value("56", &settings.target_company_id);
    fix_builder.with_value("34", count.to_string().as_str());
    //MDReqID - can be just a symbol name
    fix_builder.with_value("262", &uuid.to_string());
    //SubscriptionRequestType 1 = Snapshot + Updates
    fix_builder.with_value("263", "1");
    //Market Depth 1 = Top of Book
    fix_builder.with_value("264", "1");
    //MDUpdateType
    fix_builder.with_value("265", "0");
    //NoMDEntryTypes
    fix_builder.with_value("267", "2");
    //Bid
    fix_builder.with_value("269", "0");
    //Ask
    fix_builder.with_value("269", "1");
    //NoRelatedSym
    fix_builder.with_value("146", "1");
    //Symbol
    fix_builder.with_value("55", instrument);

    return fix_builder;
}
